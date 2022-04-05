//! Implementation for the primary directory state machine.
//!
//! There are three (active) states that a download can be in: looking
//! for a consensus ([`GetConsensusState`]), looking for certificates
//! to validate that consensus ([`GetCertsState`]), and looking for
//! microdescriptors ([`GetMicrodescsState`]).
//!
//! These states have no contact with the network, and are purely
//! reactive to other code that drives them.  See the
//! [`bootstrap`](crate::bootstrap) module for functions that actually
//! load or download directory information.

use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, SystemTime};
use time::OffsetDateTime;
use tor_error::internal;
use tor_netdir::{MdReceiver, NetDir, PartialNetDir};
use tor_netdoc::doc::netstatus::Lifetime;
use tracing::{info, warn};

use crate::event::{DirStatus, DirStatusInner};

use crate::storage::{DynStore, EXPIRATION_DEFAULTS};
use crate::{
    docmeta::{AuthCertMeta, ConsensusMeta},
    retry::DownloadSchedule,
    shared_ref::SharedMutArc,
    CacheUsage, ClientRequest, DirMgrConfig, DirState, DocId, DocumentText, Error, Readiness,
    Result,
};
use crate::{DirEvent, DocSource};
use tor_checkable::{ExternallySigned, SelfSigned, Timebound};
use tor_llcrypto::pk::rsa::RsaIdentity;
use tor_netdoc::doc::{
    microdesc::{MdDigest, Microdesc},
    netstatus::MdConsensus,
};
use tor_netdoc::{
    doc::{
        authcert::{AuthCert, AuthCertKeyIds},
        microdesc::MicrodescReader,
        netstatus::{ConsensusFlavor, UnvalidatedMdConsensus},
    },
    AllowAnnotations,
};
use tor_rtcompat::Runtime;

/// An object where we can put a usable netdir.
///
/// Note that there's only one implementation for this trait: DirMgr.
/// We make this a trait anyway to make sure that the different states
/// in this module can _only_ interact with the DirMgr through
/// modifying the NetDir and looking at the configuration.
pub(crate) trait WriteNetDir: 'static + Sync + Send {
    /// Return a DirMgrConfig to use when asked how to retry downloads,
    /// or when we need to find a list of descriptors.
    fn config(&self) -> Arc<DirMgrConfig>;

    /// Return a reference where we can write or modify a NetDir.
    fn netdir(&self) -> &SharedMutArc<NetDir>;

    /// Called to note that the consensus stored in [`Self::netdir()`] has been
    /// changed.
    fn netdir_consensus_changed(&self);

    /// Called to note that the descriptors stored in
    /// [`Self::netdir()`] have been changed.
    fn netdir_descriptors_changed(&self);

    /// Checks whether the given `netdir` is ready to replace the previous
    /// one.
    ///
    /// This is in addition to checks used when upgrading from a PartialNetDir.
    fn netdir_is_sufficient(&self, _netdir: &NetDir) -> bool {
        true
    }

    /// Called to find the current time.
    ///
    /// This is just `SystemTime::now()` in production, but for
    /// testing it is helpful to be able to mock our our current view
    /// of the time.
    fn now(&self) -> SystemTime;

    /// Return the currently configured DynFilter for this state.
    #[cfg(feature = "dirfilter")]
    fn filter(&self) -> &dyn crate::filter::DirFilter;
}

impl<R: Runtime> WriteNetDir for crate::DirMgr<R> {
    fn config(&self) -> Arc<DirMgrConfig> {
        self.config.get()
    }
    fn netdir(&self) -> &SharedMutArc<NetDir> {
        &self.netdir
    }
    fn netdir_consensus_changed(&self) {
        self.events.publish(DirEvent::NewConsensus);
    }
    fn netdir_descriptors_changed(&self) {
        self.events.publish(DirEvent::NewDescriptors);
    }
    fn netdir_is_sufficient(&self, netdir: &NetDir) -> bool {
        match &self.circmgr {
            Some(circmgr) => circmgr.netdir_is_sufficient(netdir),
            None => true, // no circmgr? then we can use anything.
        }
    }
    fn now(&self) -> SystemTime {
        self.runtime.wallclock()
    }

    #[cfg(feature = "dirfilter")]
    fn filter(&self) -> &dyn crate::filter::DirFilter {
        self.filter.as_deref().unwrap_or(&crate::filter::NilFilter)
    }
}

/// Initial state: fetching or loading a consensus directory.
#[derive(Clone, Debug)]
pub(crate) struct GetConsensusState<DM: WriteNetDir> {
    /// How should we get the consensus from the cache, if at all?
    cache_usage: CacheUsage,

    /// If present, a time after which we want our consensus to have
    /// been published.
    //
    // TODO: This is not yet used everywhere it could be.  In the future maybe
    // it should be inserted into the DocId::LatestConsensus  alternative rather
    // than being recalculated in make_consensus_request,
    after: Option<SystemTime>,

    /// If present, our next state.
    ///
    /// (This is present once we have a consensus.)
    next: Option<GetCertsState<DM>>,

    /// A list of RsaIdentity for the authorities that we believe in.
    ///
    /// No consensus can be valid unless it purports to be signed by
    /// more than half of these authorities.
    authority_ids: Vec<RsaIdentity>,

    /// A weak reference to the directory manager that wants us to
    /// fetch this information.  When this references goes away, we exit.
    writedir: Weak<DM>,
}

impl<DM: WriteNetDir> GetConsensusState<DM> {
    /// Create a new GetConsensusState from a weak reference to a
    /// directory manager and a `cache_usage` flag.
    pub(crate) fn new(writedir: Weak<DM>, cache_usage: CacheUsage) -> Result<Self> {
        let (authority_ids, after) = if let Some(writedir) = Weak::upgrade(&writedir) {
            let ids: Vec<_> = writedir
                .config()
                .authorities()
                .iter()
                .map(|auth| auth.v3ident)
                .collect();
            let after = writedir
                .netdir()
                .get()
                .map(|nd| nd.lifetime().valid_after());

            (ids, after)
        } else {
            return Err(Error::ManagerDropped);
        };
        Ok(GetConsensusState {
            cache_usage,
            after,
            next: None,
            authority_ids,
            writedir,
        })
    }
}

impl<DM: WriteNetDir> DirState for GetConsensusState<DM> {
    fn describe(&self) -> String {
        if self.next.is_some() {
            "About to fetch certificates."
        } else {
            match self.cache_usage {
                CacheUsage::CacheOnly => "Looking for a cached consensus.",
                CacheUsage::CacheOkay => "Looking for a consensus.",
                CacheUsage::MustDownload => "Downloading a consensus.",
            }
        }
        .to_string()
    }
    fn missing_docs(&self) -> Vec<DocId> {
        if self.can_advance() {
            return Vec::new();
        }
        let flavor = ConsensusFlavor::Microdesc;
        vec![DocId::LatestConsensus {
            flavor,
            cache_usage: self.cache_usage,
        }]
    }
    fn is_ready(&self, _ready: Readiness) -> bool {
        false
    }
    fn can_advance(&self) -> bool {
        self.next.is_some()
    }
    fn bootstrap_status(&self) -> DirStatus {
        if let Some(next) = &self.next {
            next.bootstrap_status()
        } else {
            DirStatusInner::NoConsensus { after: self.after }.into()
        }
    }
    fn dl_config(&self) -> Result<DownloadSchedule> {
        if let Some(wd) = Weak::upgrade(&self.writedir) {
            Ok(*wd.config().schedule().retry_consensus())
        } else {
            Err(Error::ManagerDropped)
        }
    }
    fn add_from_cache(
        &mut self,
        docs: HashMap<DocId, DocumentText>,
        _storage: Option<&Mutex<DynStore>>,
    ) -> Result<bool> {
        let text = match docs.into_iter().next() {
            None => return Ok(false),
            Some((
                DocId::LatestConsensus {
                    flavor: ConsensusFlavor::Microdesc,
                    ..
                },
                text,
            )) => text,
            _ => return Err(Error::Unwanted("Not an md consensus")),
        };

        let source = DocSource::LocalCache;
        self.add_consensus_text(source, text.as_str().map_err(Error::BadUtf8InCache)?)
            .map(|meta| meta.is_some())
    }
    fn add_from_download(
        &mut self,
        text: &str,
        _request: &ClientRequest,
        storage: Option<&Mutex<DynStore>>,
    ) -> Result<bool> {
        let source = DocSource::DirServer {};
        if let Some(meta) = self.add_consensus_text(source, text)? {
            if let Some(store) = storage {
                let mut w = store.lock().expect("Directory storage lock poisoned");
                w.store_consensus(meta, ConsensusFlavor::Microdesc, true, text)?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }
    fn advance(self: Box<Self>) -> Result<Box<dyn DirState>> {
        Ok(match self.next {
            Some(next) => Box::new(next),
            None => self,
        })
    }
    fn reset_time(&self) -> Option<SystemTime> {
        None
    }
    fn reset(self: Box<Self>) -> Result<Box<dyn DirState>> {
        Ok(self)
    }
}

impl<DM: WriteNetDir> GetConsensusState<DM> {
    /// Helper: try to set the current consensus text from an input
    /// string `text`.  Refuse it if the authorities could never be
    /// correct, or if it is ill-formed.
    fn add_consensus_text(
        &mut self,
        source: DocSource,
        text: &str,
    ) -> Result<Option<&ConsensusMeta>> {
        // Try to parse it and get its metadata.
        let (consensus_meta, unvalidated) = {
            let (signedval, remainder, parsed) =
                MdConsensus::parse(text).map_err(|e| Error::from_netdoc(source.clone(), e))?;
            #[cfg(feature = "dirfilter")]
            let parsed = if let Some(wd) = Weak::upgrade(&self.writedir) {
                wd.filter().filter_consensus(parsed)?
            } else {
                parsed
            };
            let now = current_time(&self.writedir)?;
            if let Ok(timely) = parsed.check_valid_at(&now) {
                let meta = ConsensusMeta::from_unvalidated(signedval, remainder, &timely);
                (meta, timely)
            } else {
                return Ok(None);
            }
        };

        // Check out what authorities we believe in, and see if enough
        // of them are purported to have signed this consensus.
        let n_authorities = self.authority_ids.len() as u16;
        let unvalidated = unvalidated.set_n_authorities(n_authorities);

        let id_refs: Vec<_> = self.authority_ids.iter().collect();
        if !unvalidated.authorities_are_correct(&id_refs[..]) {
            return Err(Error::UnrecognizedAuthorities);
        }

        // Make a set of all the certificates we want -- the subset of
        // those listed on the consensus that we would indeed accept as
        // authoritative.
        let desired_certs = unvalidated
            .signing_cert_ids()
            .filter(|m| self.recognizes_authority(&m.id_fingerprint))
            .collect();

        self.next = Some(GetCertsState {
            cache_usage: self.cache_usage,
            consensus_source: source,
            unvalidated,
            consensus_meta,
            missing_certs: desired_certs,
            certs: Vec::new(),
            writedir: Weak::clone(&self.writedir),
        });

        // Unwrap should be safe because `next` was just assigned
        #[allow(clippy::unwrap_used)]
        Ok(Some(&self.next.as_ref().unwrap().consensus_meta))
    }

    /// Return true if `id` is an authority identity we recognize
    fn recognizes_authority(&self, id: &RsaIdentity) -> bool {
        self.authority_ids.iter().any(|auth| auth == id)
    }
}

/// Second state: fetching or loading authority certificates.
///
/// TODO: we should probably do what C tor does, and try to use the
/// same directory that gave us the consensus.
///
/// TODO SECURITY: This needs better handling for the DOS attack where
/// we are given a bad consensus signed with fictional certificates
/// that we can never find.
#[derive(Clone, Debug)]
struct GetCertsState<DM: WriteNetDir> {
    /// The cache usage we had in mind when we began.  Used to reset.
    cache_usage: CacheUsage,
    /// Where did we get our consensus?
    consensus_source: DocSource,
    /// The consensus that we are trying to validate.
    unvalidated: UnvalidatedMdConsensus,
    /// Metadata for the consensus.
    consensus_meta: ConsensusMeta,
    /// A set of the certificate keypairs for the certificates we don't
    /// have yet.
    missing_certs: HashSet<AuthCertKeyIds>,
    /// A list of the certificates we've been able to load or download.
    certs: Vec<AuthCert>,
    /// Reference to our directory manager.
    writedir: Weak<DM>,
}

impl<DM: WriteNetDir> DirState for GetCertsState<DM> {
    fn describe(&self) -> String {
        let total = self.certs.len() + self.missing_certs.len();
        format!(
            "Downloading certificates for consensus (we are missing {}/{}).",
            self.missing_certs.len(),
            total
        )
    }
    fn missing_docs(&self) -> Vec<DocId> {
        self.missing_certs
            .iter()
            .map(|id| DocId::AuthCert(*id))
            .collect()
    }
    fn is_ready(&self, _ready: Readiness) -> bool {
        false
    }
    fn can_advance(&self) -> bool {
        self.unvalidated.key_is_correct(&self.certs[..]).is_ok()
    }
    fn bootstrap_status(&self) -> DirStatus {
        let n_certs = self.certs.len();
        let n_missing_certs = self.missing_certs.len();
        let total_certs = n_missing_certs + n_certs;
        DirStatusInner::FetchingCerts {
            lifetime: self.consensus_meta.lifetime().clone(),
            n_certs: (n_certs as u16, total_certs as u16),
        }
        .into()
    }
    fn dl_config(&self) -> Result<DownloadSchedule> {
        if let Some(wd) = Weak::upgrade(&self.writedir) {
            Ok(*wd.config().schedule().retry_certs())
        } else {
            Err(Error::ManagerDropped)
        }
    }
    fn add_from_cache(
        &mut self,
        docs: HashMap<DocId, DocumentText>,
        _storage: Option<&Mutex<DynStore>>,
    ) -> Result<bool> {
        let mut changed = false;
        // Here we iterate over the documents we want, taking them from
        // our input and remembering them.
        for id in &self.missing_docs() {
            if let Some(cert) = docs.get(id) {
                let text = cert.as_str().map_err(Error::BadUtf8InCache)?;
                let parsed = AuthCert::parse(text)
                    .map_err(|e| Error::from_netdoc(DocSource::LocalCache, e))?
                    .check_signature()?;
                let now = current_time(&self.writedir)?;
                if let Ok(cert) = parsed.check_valid_at(&now) {
                    self.missing_certs.remove(cert.key_ids());
                    self.certs.push(cert);
                    changed = true;
                } else {
                    warn!("Got a cert from our cache that we couldn't parse");
                }
            }
        }
        Ok(changed)
    }
    fn add_from_download(
        &mut self,
        text: &str,
        request: &ClientRequest,
        storage: Option<&Mutex<DynStore>>,
    ) -> Result<bool> {
        let asked_for: HashSet<_> = match request {
            ClientRequest::AuthCert(a) => a.keys().collect(),
            _ => return Err(internal!("expected an AuthCert request").into()),
        };

        let mut newcerts = Vec::new();
        for cert in AuthCert::parse_multiple(text) {
            if let Ok(parsed) = cert {
                let s = parsed
                    .within(text)
                    .expect("Certificate was not in input as expected");
                if let Ok(wellsigned) = parsed.check_signature() {
                    let now = current_time(&self.writedir)?;
                    if let Ok(timely) = wellsigned.check_valid_at(&now) {
                        newcerts.push((timely, s));
                    }
                } else {
                    // TODO: note the source.
                    warn!("Badly signed certificate received and discarded.");
                }
            } else {
                // TODO: note the source.
                warn!("Unparsable certificate received and discarded.");
            }
        }

        // Now discard any certs we didn't ask for.
        let len_orig = newcerts.len();
        newcerts.retain(|(cert, _)| asked_for.contains(cert.key_ids()));
        if newcerts.len() != len_orig {
            warn!("Discarding certificates that we didn't ask for.");
        }

        // We want to exit early if we aren't saving any certificates.
        if newcerts.is_empty() {
            return Ok(false);
        }

        if let Some(store) = storage {
            // Write the certificates to the store.
            let v: Vec<_> = newcerts[..]
                .iter()
                .map(|(cert, s)| (AuthCertMeta::from_authcert(cert), *s))
                .collect();
            let mut w = store.lock().expect("Directory storage lock poisoned");
            w.store_authcerts(&v[..])?;
        }

        // Remember the certificates in this state, and remove them
        // from our list of missing certs.
        let mut changed = false;
        for (cert, _) in newcerts {
            let ids = cert.key_ids();
            if self.missing_certs.contains(ids) {
                self.missing_certs.remove(ids);
                self.certs.push(cert);
                changed = true;
            }
        }

        Ok(changed)
    }
    fn advance(self: Box<Self>) -> Result<Box<dyn DirState>> {
        if self.can_advance() {
            let consensus_source = self.consensus_source.clone();
            let validated = self
                .unvalidated
                .check_signature(&self.certs[..])
                .map_err(|e| Error::from_netdoc(consensus_source, e))?;
            Ok(Box::new(GetMicrodescsState::new(
                self.cache_usage,
                validated,
                self.consensus_meta,
                self.writedir,
            )?))
        } else {
            Ok(self)
        }
    }
    fn reset_time(&self) -> Option<SystemTime> {
        Some(self.consensus_meta.lifetime().valid_until())
    }
    fn reset(self: Box<Self>) -> Result<Box<dyn DirState>> {
        Ok(Box::new(GetConsensusState::new(
            self.writedir,
            self.cache_usage,
        )?))
    }
}

/// Final state: we're fetching or loading microdescriptors
#[derive(Debug, Clone)]
struct GetMicrodescsState<DM: WriteNetDir> {
    /// How should we get the consensus from the cache, if at all?
    cache_usage: CacheUsage,
    /// Total number of microdescriptors listed in the consensus.
    n_microdescs: usize,
    /// The dirmgr to inform about a usable directory.
    writedir: Weak<DM>,
    /// The current status of our netdir, if it is not yet ready to become the
    /// main netdir in use for the TorClient.
    partial: Option<PendingNetDir>,
    /// Metadata for the current consensus.
    meta: ConsensusMeta,
    /// A pending list of microdescriptor digests whose
    /// "last-listed-at" times we should update.
    newly_listed: Vec<MdDigest>,
    /// A time after which we should try to replace this directory and
    /// find a new one.  Since this is randomized, we only compute it
    /// once.
    reset_time: SystemTime,
    /// If true, we should tell the storage to expire any outdated
    /// information when we finish getting a usable consensus.
    ///
    /// Only cleared for testing.
    expire_when_complete: bool,
}

/// A network directory that is not yet ready to become _the_ current network directory.
#[derive(Debug, Clone)]
enum PendingNetDir {
    /// A NetDir for which we have a consensus, but not enough microdescriptors.
    Partial(PartialNetDir),
    /// A NetDir that is "good enough to build circuits", but which we can't yet
    /// use because our `writedir` says that it isn't yet sufficient. Probably
    /// that is because we're waiting to download a microdescriptor for one or
    /// more primary guards.
    WaitingForGuards(NetDir),
}

impl MdReceiver for PendingNetDir {
    fn missing_microdescs(&self) -> Box<dyn Iterator<Item = &MdDigest> + '_> {
        match self {
            PendingNetDir::Partial(partial) => partial.missing_microdescs(),
            PendingNetDir::WaitingForGuards(netdir) => netdir.missing_microdescs(),
        }
    }

    fn add_microdesc(&mut self, md: Microdesc) -> bool {
        match self {
            PendingNetDir::Partial(partial) => partial.add_microdesc(md),
            PendingNetDir::WaitingForGuards(netdir) => netdir.add_microdesc(md),
        }
    }

    fn n_missing(&self) -> usize {
        match self {
            PendingNetDir::Partial(partial) => partial.n_missing(),
            PendingNetDir::WaitingForGuards(netdir) => netdir.n_missing(),
        }
    }
}

impl PendingNetDir {
    /// Try to move `self` as far as possible towards a complete, netdir with
    /// enough directory information (according to `writedir`).
    ///
    /// On success, return `Ok(netdir)` with a new usable [`NetDir`].  On error,
    /// return a [`PendingNetDir`] representing any progress we were able to
    /// make.
    fn upgrade<WD: WriteNetDir>(mut self, writedir: &WD) -> std::result::Result<NetDir, Self> {
        loop {
            match self {
                PendingNetDir::Partial(partial) => match partial.unwrap_if_sufficient() {
                    Ok(netdir) => {
                        self = PendingNetDir::WaitingForGuards(netdir);
                    }
                    Err(partial) => return Err(PendingNetDir::Partial(partial)),
                },
                PendingNetDir::WaitingForGuards(netdir) => {
                    if writedir.netdir_is_sufficient(&netdir) {
                        return Ok(netdir);
                    } else {
                        return Err(PendingNetDir::WaitingForGuards(netdir));
                    }
                }
            }
        }
    }
}

impl<DM: WriteNetDir> GetMicrodescsState<DM> {
    /// Create a new [`GetMicrodescsState`] from a provided
    /// microdescriptor consensus.
    fn new(
        cache_usage: CacheUsage,
        consensus: MdConsensus,
        meta: ConsensusMeta,
        writedir: Weak<DM>,
    ) -> Result<Self> {
        let reset_time = consensus.lifetime().valid_until();
        let n_microdescs = consensus.relays().len();

        let partial_dir = match Weak::upgrade(&writedir) {
            Some(wd) => {
                let config = wd.config();
                let params = config.override_net_params();
                let mut dir = PartialNetDir::new(consensus, Some(params));
                if let Some(old_dir) = wd.netdir().get() {
                    dir.fill_from_previous_netdir(&old_dir);
                }
                dir
            }
            None => return Err(Error::ManagerDropped),
        };

        let mut result = GetMicrodescsState {
            cache_usage,
            n_microdescs,
            writedir,
            partial: Some(PendingNetDir::Partial(partial_dir)),
            meta,
            newly_listed: Vec::new(),
            reset_time,
            expire_when_complete: true,
        };

        result.consider_upgrade();
        Ok(result)
    }

    /// Add a bunch of microdescriptors to the in-progress netdir.
    ///
    /// Return true if the netdir has just become usable.
    fn register_microdescs<I>(&mut self, mds: I) -> bool
    where
        I: IntoIterator<Item = Microdesc>,
    {
        #[cfg(feature = "dirfilter")]
        let mds: Vec<Microdesc> = if let Some(wd) = Weak::upgrade(&self.writedir) {
            mds.into_iter()
                .filter_map(|m| wd.filter().filter_md(m).ok())
                .collect()
        } else {
            mds.into_iter().collect()
        };
        if let Some(p) = &mut self.partial {
            for md in mds {
                self.newly_listed.push(*md.digest());
                p.add_microdesc(md);
            }
            return self.consider_upgrade();
        } else if let Some(wd) = Weak::upgrade(&self.writedir) {
            let _ = wd.netdir().mutate(|netdir| {
                for md in mds {
                    netdir.add_microdesc(md);
                }
                wd.netdir_descriptors_changed();
                Ok(())
            });
        }
        false
    }

    /// Check whether this netdir we're building has _just_ become
    /// usable when it was not previously usable.  If so, tell the
    /// dirmgr about it and return true; otherwise return false.
    fn consider_upgrade(&mut self) -> bool {
        if let Some(p) = self.partial.take() {
            if let Some(wd) = Weak::upgrade(&self.writedir) {
                match p.upgrade(wd.as_ref()) {
                    Ok(mut netdir) => {
                        self.reset_time = pick_download_time(netdir.lifetime());
                        // We re-set the parameters here, in case they have been
                        // reconfigured.
                        netdir.replace_overridden_parameters(wd.config().override_net_params());
                        wd.netdir().replace(netdir);
                        wd.netdir_consensus_changed();
                        wd.netdir_descriptors_changed();
                        return true;
                    }
                    Err(pending) => self.partial = Some(pending),
                }
            }
        }
        false
    }

    /// Mark the consensus that we're getting MDs for as non-pending in the
    /// storage.
    ///
    /// Called when a consensus is no longer pending.
    fn mark_consensus_usable(&self, storage: Option<&Mutex<DynStore>>) -> Result<()> {
        if let Some(store) = storage {
            let mut store = store.lock().expect("Directory storage lock poisoned");
            info!("Marked consensus usable.");
            store.mark_consensus_usable(&self.meta)?;
            // Now that a consensus is usable, older consensuses may
            // need to expire.
            if self.expire_when_complete {
                store.expire_all(&EXPIRATION_DEFAULTS)?;
            }
        }
        Ok(())
    }
}

impl<DM: WriteNetDir> GetMicrodescsState<DM> {
    /// Try to obtain info from an inner `MdReceiver`
    ///
    /// Either finds an inner `MdReceiver` and calls `f` on it, or returns `default()`.
    ///
    /// Used for missing microdescs.
    fn with_mdreceiver_for_missing<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&dyn MdReceiver) -> T,
        T: Default,
    {
        if let Some(partial) = &self.partial {
            return f(partial);
        } else if let Some(wd) = Weak::upgrade(&self.writedir) {
            if let Some(nd) = wd.netdir().get() {
                return f(nd.as_ref());
            }
        }
        Default::default()
    }

    /// Number of missing microdescriptors, or 0
    ///
    /// Can return 0 if we don't have that information.
    fn n_missing(&self) -> usize {
        self.with_mdreceiver_for_missing(|d| d.n_missing())
    }
}

impl<DM: WriteNetDir> DirState for GetMicrodescsState<DM> {
    fn describe(&self) -> String {
        format!(
            "Downloading microdescriptors (we are missing {}).",
            self.n_missing()
        )
    }
    fn missing_docs(&self) -> Vec<DocId> {
        self.with_mdreceiver_for_missing(|d| {
            d.missing_microdescs()
                .map(|d| DocId::Microdesc(*d))
                .collect()
        })
    }
    fn is_ready(&self, ready: Readiness) -> bool {
        match ready {
            Readiness::Complete => self.n_missing() == 0,
            Readiness::Usable => self.partial.is_none(),
        }
    }
    fn can_advance(&self) -> bool {
        false
    }
    fn bootstrap_status(&self) -> DirStatus {
        let n_present = self.n_microdescs - self.n_missing();
        DirStatusInner::Validated {
            lifetime: self.meta.lifetime().clone(),
            n_mds: (n_present as u32, self.n_microdescs as u32),
            usable: self.is_ready(Readiness::Usable),
        }
        .into()
    }
    fn dl_config(&self) -> Result<DownloadSchedule> {
        if let Some(wd) = Weak::upgrade(&self.writedir) {
            Ok(*wd.config().schedule().retry_microdescs())
        } else {
            Err(Error::ManagerDropped)
        }
    }
    fn add_from_cache(
        &mut self,
        docs: HashMap<DocId, DocumentText>,
        storage: Option<&Mutex<DynStore>>,
    ) -> Result<bool> {
        let mut microdescs = Vec::new();
        for (id, text) in docs {
            if let DocId::Microdesc(digest) = id {
                if let Ok(md) = Microdesc::parse(text.as_str().map_err(Error::BadUtf8InCache)?) {
                    if md.digest() == &digest {
                        microdescs.push(md);
                        continue;
                    }
                }
                warn!("Found a mismatched microdescriptor in cache; ignoring");
            }
        }

        let changed = !microdescs.is_empty();
        if self.register_microdescs(microdescs) {
            // Just stopped being pending.
            self.mark_consensus_usable(storage)?;
            // We can save a lot of ram this way, though we don't want to do it
            // so often.  As a compromise we call `shrink_to_fit at most twice
            // per consensus: once when we're ready to use, and once when we are
            // complete.
        }

        Ok(changed)
    }

    fn add_from_download(
        &mut self,
        text: &str,
        request: &ClientRequest,
        storage: Option<&Mutex<DynStore>>,
    ) -> Result<bool> {
        let requested: HashSet<_> = if let ClientRequest::Microdescs(req) = request {
            req.digests().collect()
        } else {
            return Err(internal!("expected a microdesc request").into());
        };
        let mut new_mds = Vec::new();
        for anno in MicrodescReader::new(text, &AllowAnnotations::AnnotationsNotAllowed).flatten() {
            let txt = anno
                .within(text)
                .expect("annotation not from within text as expected");
            let md = anno.into_microdesc();
            if !requested.contains(md.digest()) {
                warn!(
                    "Received microdescriptor we did not ask for: {:?}",
                    md.digest()
                );
                continue;
            }
            new_mds.push((txt, md));
        }

        let mark_listed = self.meta.lifetime().valid_after();
        if let Some(store) = storage {
            let mut s = store
                .lock()
                //.get_mut()
                .expect("Directory storage lock poisoned");
            if !self.newly_listed.is_empty() {
                s.update_microdescs_listed(&self.newly_listed, mark_listed)?;
                self.newly_listed.clear();
            }
            if !new_mds.is_empty() {
                s.store_microdescs(
                    &new_mds
                        .iter()
                        .map(|(text, md)| (*text, md.digest()))
                        .collect::<Vec<_>>(),
                    mark_listed,
                )?;
            }
        }
        if self.register_microdescs(new_mds.into_iter().map(|(_, md)| md)) {
            // Just stopped being pending.
            self.mark_consensus_usable(storage)?;
        }
        Ok(true)
    }
    fn advance(self: Box<Self>) -> Result<Box<dyn DirState>> {
        Ok(self)
    }
    fn reset_time(&self) -> Option<SystemTime> {
        Some(self.reset_time)
    }
    fn reset(self: Box<Self>) -> Result<Box<dyn DirState>> {
        let cache_usage = if self.cache_usage == CacheUsage::CacheOnly {
            // Cache only means we can't ever download.
            CacheUsage::CacheOnly
        } else if self.is_ready(Readiness::Usable) {
            // If we managed to bootstrap a usable consensus, then we won't
            // accept our next consensus from the cache.
            CacheUsage::MustDownload
        } else {
            // If we didn't manage to bootstrap a usable consensus, then we can
            // indeed try again with the one in the cache.
            // TODO(nickm) is this right?
            CacheUsage::CacheOkay
        };
        Ok(Box::new(GetConsensusState::new(
            self.writedir,
            cache_usage,
        )?))
    }
}

/// Choose a random download time to replace a consensus whose lifetime
/// is `lifetime`.
fn pick_download_time(lifetime: &Lifetime) -> SystemTime {
    let (lowbound, uncertainty) = client_download_range(lifetime);
    let zero = Duration::new(0, 0);
    let t = lowbound + rand::thread_rng().gen_range(zero..uncertainty);
    info!("The current consensus is fresh until {}, and valid until {}. I've picked {} as the earliest time to replace it.",
          OffsetDateTime::from(lifetime.fresh_until()),
          OffsetDateTime::from(lifetime.valid_until()),
          OffsetDateTime::from(t));
    t
}

/// Based on the lifetime for a consensus, return the time range during which
/// clients should fetch the next one.
fn client_download_range(lt: &Lifetime) -> (SystemTime, Duration) {
    let valid_after = lt.valid_after();
    let fresh_until = lt.fresh_until();
    let valid_until = lt.valid_until();
    let voting_interval = fresh_until
        .duration_since(valid_after)
        .expect("valid-after must precede fresh-until");
    let whole_lifetime = valid_until
        .duration_since(valid_after)
        .expect("valid-after must precede valid-until");

    // From dir-spec:
    // "This time is chosen uniformly at random from the interval
    // between the time 3/4 into the first interval after the
    // consensus is no longer fresh, and 7/8 of the time remaining
    // after that before the consensus is invalid."
    let lowbound = voting_interval + (voting_interval * 3) / 4;
    let remainder = whole_lifetime - lowbound;
    let uncertainty = (remainder * 7) / 8;

    (valid_after + lowbound, uncertainty)
}

/// Helper: call `now` on a Weak<WriteNetDir>.
fn current_time<DM: WriteNetDir>(writedir: &Weak<DM>) -> Result<SystemTime> {
    if let Some(writedir) = Weak::upgrade(writedir) {
        Ok(writedir.now())
    } else {
        Err(Error::ManagerDropped)
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::cognitive_complexity)]
    use super::*;
    use crate::{Authority, DownloadScheduleConfig};
    use std::convert::TryInto;
    use std::sync::{
        atomic::{self, AtomicBool},
        Arc,
    };
    use tempfile::TempDir;
    use time::macros::datetime;
    use tor_netdoc::doc::authcert::AuthCertKeyIds;

    #[test]
    fn download_schedule() {
        let va = datetime!(2008-08-02 20:00 UTC).into();
        let fu = datetime!(2008-08-02 21:00 UTC).into();
        let vu = datetime!(2008-08-02 23:00 UTC).into();
        let lifetime = Lifetime::new(va, fu, vu).unwrap();

        let expected_start: SystemTime = datetime!(2008-08-02 21:45 UTC).into();
        let expected_range = Duration::from_millis((75 * 60 * 1000) * 7 / 8);

        let (start, range) = client_download_range(&lifetime);
        assert_eq!(start, expected_start);
        assert_eq!(range, expected_range);

        for _ in 0..100 {
            let when = pick_download_time(&lifetime);
            assert!(when > va);
            assert!(when >= expected_start);
            assert!(when < vu);
            assert!(when <= expected_start + range);
        }
    }

    /// Makes a memory-backed storage.
    fn temp_store() -> (TempDir, Mutex<DynStore>) {
        let tempdir = TempDir::new().unwrap();

        let store = crate::storage::SqliteStore::from_path(tempdir.path(), false).unwrap();

        (tempdir, Mutex::new(Box::new(store)))
    }

    struct DirRcv {
        cfg: Arc<DirMgrConfig>,
        netdir: SharedMutArc<NetDir>,
        consensus_changed: AtomicBool,
        descriptors_changed: AtomicBool,
        now: SystemTime,
    }

    impl DirRcv {
        fn new(now: SystemTime, authorities: Option<Vec<Authority>>) -> Self {
            let mut netcfg = crate::NetworkConfig::builder();
            netcfg.fallback_caches(vec![]);
            if let Some(a) = authorities {
                netcfg.authorities(a);
            }
            let cfg = DirMgrConfig {
                cache_path: "/we_will_never_use_this/".into(),
                network_config: netcfg.build().unwrap(),
                ..Default::default()
            };
            let cfg = Arc::new(cfg);
            DirRcv {
                now,
                cfg,
                netdir: Default::default(),
                consensus_changed: false.into(),
                descriptors_changed: false.into(),
            }
        }
    }

    impl WriteNetDir for DirRcv {
        fn config(&self) -> Arc<DirMgrConfig> {
            Arc::clone(&self.cfg)
        }
        fn netdir(&self) -> &SharedMutArc<NetDir> {
            &self.netdir
        }
        fn netdir_consensus_changed(&self) {
            self.consensus_changed.store(true, atomic::Ordering::SeqCst);
        }
        fn netdir_descriptors_changed(&self) {
            self.descriptors_changed
                .store(true, atomic::Ordering::SeqCst);
        }
        fn now(&self) -> SystemTime {
            self.now
        }
        #[cfg(feature = "dirfilter")]
        fn filter(&self) -> &dyn crate::filter::DirFilter {
            &crate::filter::NilFilter
        }
    }

    // Test data
    const CONSENSUS: &str = include_str!("../testdata/mdconsensus1.txt");
    const CONSENSUS2: &str = include_str!("../testdata/mdconsensus2.txt");
    const AUTHCERT_5696: &str = include_str!("../testdata/cert-5696.txt");
    const AUTHCERT_5A23: &str = include_str!("../testdata/cert-5A23.txt");
    #[allow(unused)]
    const AUTHCERT_7C47: &str = include_str!("../testdata/cert-7C47.txt");
    fn test_time() -> SystemTime {
        datetime!(2020-08-07 12:42:45 UTC).into()
    }
    fn rsa(s: &str) -> RsaIdentity {
        RsaIdentity::from_hex(s).unwrap()
    }
    fn test_authorities() -> Vec<Authority> {
        fn a(s: &str) -> Authority {
            Authority::builder()
                .name("ignore")
                .v3ident(rsa(s))
                .build()
                .unwrap()
        }
        vec![
            a("5696AB38CB3852AFA476A5C07B2D4788963D5567"),
            a("5A23BA701776C9C1AB1C06E734E92AB3D5350D64"),
            // This is an authority according to the consensus, but we'll
            // pretend we don't recognize it, to make sure that we
            // don't fetch or accept it.
            // a("7C47DCB4A90E2C2B7C7AD27BD641D038CF5D7EBE"),
        ]
    }
    fn authcert_id_5696() -> AuthCertKeyIds {
        AuthCertKeyIds {
            id_fingerprint: rsa("5696ab38cb3852afa476a5c07b2d4788963d5567"),
            sk_fingerprint: rsa("f6ed4aa64d83caede34e19693a7fcf331aae8a6a"),
        }
    }
    fn authcert_id_5a23() -> AuthCertKeyIds {
        AuthCertKeyIds {
            id_fingerprint: rsa("5a23ba701776c9c1ab1c06e734e92ab3d5350d64"),
            sk_fingerprint: rsa("d08e965cc6dcb6cb6ed776db43e616e93af61177"),
        }
    }
    // remember, we're saying that we don't recognize this one as an authority.
    fn authcert_id_7c47() -> AuthCertKeyIds {
        AuthCertKeyIds {
            id_fingerprint: rsa("7C47DCB4A90E2C2B7C7AD27BD641D038CF5D7EBE"),
            sk_fingerprint: rsa("D3C013E0E6C82E246090D1C0798B75FCB7ACF120"),
        }
    }
    fn microdescs() -> HashMap<MdDigest, String> {
        const MICRODESCS: &str = include_str!("../testdata/microdescs.txt");
        let text = MICRODESCS;
        MicrodescReader::new(text, &AllowAnnotations::AnnotationsNotAllowed)
            .map(|res| {
                let anno = res.unwrap();
                let text = anno.within(text).unwrap();
                let md = anno.into_microdesc();
                (*md.digest(), text.to_owned())
            })
            .collect()
    }

    #[test]
    fn get_consensus_state() {
        let rcv = Arc::new(DirRcv::new(test_time(), None));

        let (_tempdir, store) = temp_store();

        let mut state =
            GetConsensusState::new(Arc::downgrade(&rcv), CacheUsage::CacheOkay).unwrap();

        // Is description okay?
        assert_eq!(&state.describe(), "Looking for a consensus.");

        // Basic properties: without a consensus it is not ready to advance.
        assert!(!state.can_advance());
        assert!(!state.is_ready(Readiness::Complete));
        assert!(!state.is_ready(Readiness::Usable));

        // Basic properties: it doesn't want to reset.
        assert!(state.reset_time().is_none());

        // Its starting DirStatus is "fetching a consensus".
        assert_eq!(state.bootstrap_status().to_string(), "fetching a consensus");

        // Download configuration is simple: only 1 request can be done in
        // parallel.  It uses a consensus retry schedule.
        let retry = state.dl_config().unwrap();
        assert_eq!(&retry, DownloadScheduleConfig::default().retry_consensus());

        // Do we know what we want?
        let docs = state.missing_docs();
        assert_eq!(docs.len(), 1);
        let docid = docs[0];

        assert!(matches!(
            docid,
            DocId::LatestConsensus {
                flavor: ConsensusFlavor::Microdesc,
                cache_usage: CacheUsage::CacheOkay,
            }
        ));

        // Now suppose that we get some complete junk from a download.
        let req = tor_dirclient::request::ConsensusRequest::new(ConsensusFlavor::Microdesc);
        let req = crate::docid::ClientRequest::Consensus(req);
        let outcome = state.add_from_download("this isn't a consensus", &req, Some(&store));
        assert!(matches!(outcome, Err(Error::NetDocError { .. })));
        // make sure it wasn't stored...
        assert!(store
            .lock()
            .unwrap()
            .latest_consensus(ConsensusFlavor::Microdesc, None)
            .unwrap()
            .is_none());

        // Now try again, with a real consensus... but the wrong authorities.
        let outcome = state.add_from_download(CONSENSUS, &req, Some(&store));
        assert!(matches!(outcome, Err(Error::UnrecognizedAuthorities)));
        assert!(store
            .lock()
            .unwrap()
            .latest_consensus(ConsensusFlavor::Microdesc, None)
            .unwrap()
            .is_none());

        // Great. Change the receiver to use a configuration where these test
        // authorities are recognized.
        let rcv = Arc::new(DirRcv::new(test_time(), Some(test_authorities())));

        let mut state =
            GetConsensusState::new(Arc::downgrade(&rcv), CacheUsage::CacheOkay).unwrap();
        let outcome = state.add_from_download(CONSENSUS, &req, Some(&store));
        assert!(outcome.unwrap());
        assert!(store
            .lock()
            .unwrap()
            .latest_consensus(ConsensusFlavor::Microdesc, None)
            .unwrap()
            .is_some());

        // And with that, we should be asking for certificates
        assert!(state.can_advance());
        assert_eq!(&state.describe(), "About to fetch certificates.");
        assert_eq!(state.missing_docs(), Vec::new());
        let next = Box::new(state).advance().unwrap();
        assert_eq!(
            &next.describe(),
            "Downloading certificates for consensus (we are missing 2/2)."
        );

        // Try again, but this time get the state from the cache.
        let rcv = Arc::new(DirRcv::new(test_time(), Some(test_authorities())));
        let mut state =
            GetConsensusState::new(Arc::downgrade(&rcv), CacheUsage::CacheOkay).unwrap();
        let text: crate::storage::InputString = CONSENSUS.to_owned().into();
        let map = vec![(docid, text.into())].into_iter().collect();
        let outcome = state.add_from_cache(map, None);
        assert!(outcome.unwrap());
        assert!(state.can_advance());
    }

    #[test]
    fn get_certs_state() {
        /// Construct a GetCertsState with our test data
        fn new_getcerts_state() -> (Arc<DirRcv>, Box<dyn DirState>) {
            let rcv = Arc::new(DirRcv::new(test_time(), Some(test_authorities())));
            let mut state =
                GetConsensusState::new(Arc::downgrade(&rcv), CacheUsage::CacheOkay).unwrap();
            let req = tor_dirclient::request::ConsensusRequest::new(ConsensusFlavor::Microdesc);
            let req = crate::docid::ClientRequest::Consensus(req);
            let outcome = state.add_from_download(CONSENSUS, &req, None);
            assert!(outcome.unwrap());
            (rcv, Box::new(state).advance().unwrap())
        }

        let (_tempdir, store) = temp_store();
        let (_rcv, mut state) = new_getcerts_state();
        // Basic properties: description, status, reset time.
        assert_eq!(
            &state.describe(),
            "Downloading certificates for consensus (we are missing 2/2)."
        );
        assert!(!state.can_advance());
        assert!(!state.is_ready(Readiness::Complete));
        assert!(!state.is_ready(Readiness::Usable));
        let consensus_expires = datetime!(2020-08-07 12:43:20 UTC).into();
        assert_eq!(state.reset_time(), Some(consensus_expires));
        let retry = state.dl_config().unwrap();
        assert_eq!(&retry, DownloadScheduleConfig::default().retry_certs());

        // Bootstrap status okay?
        assert_eq!(
            state.bootstrap_status().to_string(),
            "fetching authority certificates (0/2)"
        );

        // Check that we get the right list of missing docs.
        let missing = state.missing_docs();
        assert_eq!(missing.len(), 2); // We are missing two certificates.
        assert!(missing.contains(&DocId::AuthCert(authcert_id_5696())));
        assert!(missing.contains(&DocId::AuthCert(authcert_id_5a23())));
        // we don't ask for this one because we don't recognize its authority
        assert!(!missing.contains(&DocId::AuthCert(authcert_id_7c47())));

        // Add one from the cache; make sure the list is still right
        let text1: crate::storage::InputString = AUTHCERT_5696.to_owned().into();
        // let text2: crate::storage::InputString = AUTHCERT_5A23.to_owned().into();
        let docs = vec![(DocId::AuthCert(authcert_id_5696()), text1.into())]
            .into_iter()
            .collect();
        let outcome = state.add_from_cache(docs, None);
        assert!(outcome.unwrap()); // no error, and something changed.
        assert!(!state.can_advance()); // But we aren't done yet.
        let missing = state.missing_docs();
        assert_eq!(missing.len(), 1); // Now we're only missing one!
        assert!(missing.contains(&DocId::AuthCert(authcert_id_5a23())));
        assert_eq!(
            state.bootstrap_status().to_string(),
            "fetching authority certificates (1/2)"
        );

        // Now try to add the other from a download ... but fail
        // because we didn't ask for it.
        let mut req = tor_dirclient::request::AuthCertRequest::new();
        req.push(authcert_id_5696()); // it's the wrong id.
        let req = ClientRequest::AuthCert(req);
        let outcome = state.add_from_download(AUTHCERT_5A23, &req, Some(&store));
        assert!(!outcome.unwrap()); // no error, but nothing changed.
        let missing2 = state.missing_docs();
        assert_eq!(missing, missing2); // No change.
        assert!(store
            .lock()
            .unwrap()
            .authcerts(&[authcert_id_5a23()])
            .unwrap()
            .is_empty());

        // Now try to add the other from a download ... for real!
        let mut req = tor_dirclient::request::AuthCertRequest::new();
        req.push(authcert_id_5a23()); // Right idea this time!
        let req = ClientRequest::AuthCert(req);
        let outcome = state.add_from_download(AUTHCERT_5A23, &req, Some(&store));
        assert!(outcome.unwrap()); // No error, _and_ something changed!
        let missing3 = state.missing_docs();
        assert!(missing3.is_empty());
        assert!(state.can_advance());
        assert!(!store
            .lock()
            .unwrap()
            .authcerts(&[authcert_id_5a23()])
            .unwrap()
            .is_empty());

        let next = state.advance().unwrap();
        assert_eq!(
            &next.describe(),
            "Downloading microdescriptors (we are missing 6)."
        );

        // If we start from scratch and reset, we're back in GetConsensus.
        let (_rcv, state) = new_getcerts_state();
        let state = state.reset().unwrap();
        assert_eq!(&state.describe(), "Looking for a consensus.");

        // TODO: I'd like even more tests to make sure that we never
        // accept a certificate for an authority we don't believe in.
    }

    #[test]
    fn get_microdescs_state() {
        /// Construct a GetCertsState with our test data
        fn new_getmicrodescs_state() -> (Arc<DirRcv>, GetMicrodescsState<DirRcv>) {
            let rcv = Arc::new(DirRcv::new(test_time(), Some(test_authorities())));
            let (signed, rest, consensus) = MdConsensus::parse(CONSENSUS2).unwrap();
            let consensus = consensus
                .dangerously_assume_timely()
                .dangerously_assume_wellsigned();
            let meta = ConsensusMeta::from_consensus(signed, rest, &consensus);
            let state = GetMicrodescsState::new(
                CacheUsage::CacheOkay,
                consensus,
                meta,
                Arc::downgrade(&rcv),
            )
            .unwrap();

            (rcv, state)
        }
        fn d64(s: &str) -> MdDigest {
            base64::decode(s).unwrap().try_into().unwrap()
        }

        // If we start from scratch and reset, we're back in GetConsensus.
        let (_rcv, state) = new_getmicrodescs_state();
        let state = Box::new(state).reset().unwrap();
        assert_eq!(&state.describe(), "Looking for a consensus.");

        // Check the basics.
        let (_rcv, mut state) = new_getmicrodescs_state();
        assert_eq!(
            &state.describe(),
            "Downloading microdescriptors (we are missing 4)."
        );
        assert!(!state.can_advance());
        assert!(!state.is_ready(Readiness::Complete));
        assert!(!state.is_ready(Readiness::Usable));
        {
            let reset_time = state.reset_time().unwrap();
            let fresh_until: SystemTime = datetime!(2021-10-27 21:27:00 UTC).into();
            let valid_until: SystemTime = datetime!(2021-10-27 21:27:20 UTC).into();
            assert!(reset_time >= fresh_until);
            assert!(reset_time <= valid_until);
        }
        let retry = state.dl_config().unwrap();
        assert_eq!(&retry, DownloadScheduleConfig::default().retry_microdescs());
        assert_eq!(
            state.bootstrap_status().to_string(),
            "fetching microdescriptors (0/4)"
        );

        // Now check whether we're missing all the right microdescs.
        let missing = state.missing_docs();
        let md_text = microdescs();
        assert_eq!(missing.len(), 4);
        assert_eq!(md_text.len(), 4);
        let md1 = d64("LOXRj8YZP0kwpEAsYOvBZWZWGoWv5b/Bp2Mz2Us8d8g");
        let md2 = d64("iOhVp33NyZxMRDMHsVNq575rkpRViIJ9LN9yn++nPG0");
        let md3 = d64("/Cd07b3Bl0K0jX2/1cAvsYXJJMi5d8UBU+oWKaLxoGo");
        let md4 = d64("z+oOlR7Ga6cg9OoC/A3D3Ey9Rtc4OldhKlpQblMfQKo");
        for md_digest in [md1, md2, md3, md4] {
            assert!(missing.contains(&DocId::Microdesc(md_digest)));
            assert!(md_text.contains_key(&md_digest));
        }

        // Try adding a microdesc from the cache.
        let (_tempdir, store) = temp_store();
        let doc1: crate::storage::InputString = md_text.get(&md1).unwrap().clone().into();
        let docs = vec![(DocId::Microdesc(md1), doc1.into())]
            .into_iter()
            .collect();
        let outcome = state.add_from_cache(docs, Some(&store));
        assert!(outcome.unwrap()); // successfully loaded one MD.
        assert!(!state.can_advance());
        assert!(!state.is_ready(Readiness::Complete));
        assert!(!state.is_ready(Readiness::Usable));

        // Now we should be missing 3.
        let missing = state.missing_docs();
        assert_eq!(missing.len(), 3);
        assert!(!missing.contains(&DocId::Microdesc(md1)));
        assert_eq!(
            state.bootstrap_status().to_string(),
            "fetching microdescriptors (1/4)"
        );

        // Try adding the rest as if from a download.
        let mut req = tor_dirclient::request::MicrodescRequest::new();
        // Clear this flag so that the test consensus won't expire the moment
        // we're done.
        state.expire_when_complete = false;
        let mut response = "".to_owned();
        for md_digest in [md2, md3, md4] {
            response.push_str(md_text.get(&md_digest).unwrap());
            req.push(md_digest);
        }
        let req = ClientRequest::Microdescs(req);
        let outcome = state.add_from_download(response.as_str(), &req, Some(&store));
        assert!(outcome.unwrap()); // successfully loaded MDs
        assert!(state.is_ready(Readiness::Complete));
        assert!(state.is_ready(Readiness::Usable));
        assert_eq!(
            store
                .lock()
                .unwrap()
                .microdescs(&[md2, md3, md4])
                .unwrap()
                .len(),
            3
        );

        let missing = state.missing_docs();
        assert!(missing.is_empty());
    }
}
