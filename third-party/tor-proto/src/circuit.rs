//! Multi-hop paths over the Tor network.
//!
//! Right now, we only implement "client circuits" -- also sometimes
//! called "origin circuits".  A client circuit is one that is
//! constructed by this Tor instance, and used in its own behalf to
//! send data over the Tor network.
//!
//! Each circuit has multiple hops over the Tor network: each hop
//! knows only the hop before and the hop after.  The client shares a
//! separate set of keys with each hop.
//!
//! To build a circuit, first create a [crate::channel::Channel], then
//! call its [crate::channel::Channel::new_circ] method.  This yields
//! a [PendingClientCirc] object that won't become live until you call
//! one of the methods that extends it to its first hop.  After you've
//! done that, you can call [ClientCirc::extend_ntor] on the circuit to
//! build it into a multi-hop circuit.  Finally, you can use
//! [ClientCirc::begin_stream] to get a Stream object that can be used
//! for anonymized data.
//!
//! # Implementation
//!
//! Each open circuit has a corresponding Reactor object that runs in
//! an asynchronous task, and manages incoming cells from the
//! circuit's upstream channel.  These cells are either RELAY cells or
//! DESTROY cells.  DESTROY cells are handled immediately.
//! RELAY cells are either for a particular stream, in which case they
//! get forwarded to a RawCellStream object, or for no particular stream,
//! in which case they are considered "meta" cells (like EXTENDED2)
//! that should only get accepted if something is waiting for them.
//!
//! # Limitations
//!
//! This is client-only.
//!
//! There's one big mutex on the whole circuit: the reactor needs to hold
//! it to process a cell, and streams need to hold it to send.
//!
//! There is no flow-control or rate-limiting or fairness.

pub(crate) mod celltypes;
pub(crate) mod halfcirc;
mod halfstream;
#[cfg(feature = "hs-common")]
pub mod handshake;
#[cfg(feature = "send-control-msg")]
mod msghandler;
mod path;
pub(crate) mod reactor;
pub(crate) mod sendme;
mod streammap;
mod unique_id;

use crate::channel::Channel;
use crate::circuit::celltypes::*;
use crate::circuit::reactor::{
    CircuitHandshake, CtrlMsg, Reactor, RECV_WINDOW_INIT, STREAM_READER_BUFFER,
};
pub use crate::circuit::unique_id::UniqId;
use crate::crypto::cell::HopNum;
use crate::stream::{
    AnyCmdChecker, DataCmdChecker, DataStream, ResolveCmdChecker, ResolveStream, StreamParameters,
    StreamReader,
};
use crate::{Error, ResolveError, Result};
use tor_cell::{
    chancell::{self, msg::AnyChanMsg, CircId},
    relaycell::msg::{AnyRelayMsg, Begin, Resolve, Resolved, ResolvedVal},
};

use tor_error::{bad_api_usage, internal, into_internal};
use tor_linkspec::{CircTarget, LinkSpecType, OwnedChanTarget, RelayIdType};

use futures::channel::{mpsc, oneshot};

use crate::circuit::sendme::StreamRecvWindow;
use futures::SinkExt;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use tor_cell::relaycell::StreamId;
// use std::time::Duration;

use crate::crypto::handshake::ntor::NtorPublicKey;
pub use path::{Path, PathEntry};

/// The size of the buffer for communication between `ClientCirc` and its reactor.
pub const CIRCUIT_BUFFER_SIZE: usize = 128;

#[cfg(feature = "send-control-msg")]
#[cfg_attr(docsrs, doc(cfg(feature = "send-control-msg")))]
pub use {msghandler::MsgHandler, reactor::MetaCellDisposition};

#[derive(Debug)]
/// A circuit that we have constructed over the Tor network.
///
/// # Circuit life cycle
///
/// `ClientCirc`s are created in an initially unusable state using [`Channel::new_circ`],
/// which returns a [`PendingClientCirc`].  To get a real (one-hop) circuit from
/// one of these, you invoke one of its `create_firsthop` methods (currently
/// [`create_firsthop_fast()`](PendingClientCirc::create_firsthop_fast) or
/// [`create_firsthop_ntor()`](PendingClientCirc::create_firsthop_ntor)).
/// Then, to add more hops to the circuit, you can call
/// [`extend_ntor()`](ClientCirc::extend_ntor) on it.
///
/// For higher-level APIs, see the `tor-circmgr` crate: the ones here in
/// `tor-proto` are probably not what you need.
///
/// After a circuit is created, it will persist until it is closed in one of
/// five ways:
///    1. A remote error occurs.
///    2. Some hop on the circuit sends a `DESTROY` message to tear down the
///       circuit.
///    3. The circuit's channel is closed.
///    4. Someone calls [`ClientCirc::terminate`] on the circuit.
///    5. The last reference to the `ClientCirc` is dropped. (Note that every stream
///       on a `ClientCirc` keeps a reference to it, which will in turn keep the
///       circuit from closing until all those streams have gone away.)
///
/// Note that in cases 1-4 the [`ClientCirc`] object itself will still exist: it
/// will just be unusable for most purposes.  Most operations on it will fail
/// with an error.
//
// Effectively, this struct contains two Arcs: one for `path` and one for
// `control` (which surely has something Arc-like in it).  We cannot unify
// these by putting a single Arc around the whole struct, and passing
// an Arc strong reference to the `Reactor`, because then `control` would
// not be dropped when the last user of the circuit goes away.  We could
// make the reactor have a weak reference but weak references are more
// expensive to dereference.
//
// Because of the above, cloning this struct is always going to involve
// two atomic refcount changes/checks.  Wrapping it in another Arc would
// be overkill.

pub struct ClientCirc {
    /// Mutable state shared with the `Reactor`.
    mutable: Arc<Mutex<MutableState>>,
    /// A unique identifier for this circuit.
    unique_id: UniqId,
    /// Channel to send control messages to the reactor.
    control: mpsc::UnboundedSender<CtrlMsg>,
    /// The channel that this ClientCirc is connected to and using to speak with
    /// its first hop.
    ///
    /// # Warning
    ///
    /// Don't use this field to send or receive any data, or perform any network
    /// operations for this circuit!  All network operations should be done by
    /// the circuit reactor.
    channel: Channel,
    /// For testing purposes: the CircId, for use in peek_circid().
    #[cfg(test)]
    circid: CircId,
}

/// Mutable state shared by [`ClientCirc`] and [`Reactor`].
#[derive(Debug)]
struct MutableState {
    /// Information about this circuit's path.
    ///
    /// This is stored in an Arc so that we can cheaply give a copy of it to
    /// client code; when we need to add a hop (which is less frequent) we use
    /// [`Arc::make_mut()`].
    path: Arc<path::Path>,
}

/// A ClientCirc that needs to send a create cell and receive a created* cell.
///
/// To use one of these, call create_firsthop_fast() or create_firsthop_ntor()
/// to negotiate the cryptographic handshake with the first hop.
pub struct PendingClientCirc {
    /// A oneshot receiver on which we'll receive a CREATED* cell,
    /// or a DESTROY cell.
    recvcreated: oneshot::Receiver<CreateResponse>,
    /// The ClientCirc object that we can expose on success.
    circ: Arc<ClientCirc>,
}

/// Description of the network's current rules for building circuits.
#[derive(Clone, Debug)]
pub struct CircParameters {
    /// Initial value to use for our outbound circuit-level windows.
    initial_send_window: u16,
    /// Whether we should include ed25519 identities when we send
    /// EXTEND2 cells.
    extend_by_ed25519_id: bool,
}

impl Default for CircParameters {
    fn default() -> CircParameters {
        CircParameters {
            initial_send_window: 1000,
            extend_by_ed25519_id: true,
        }
    }
}

impl CircParameters {
    /// Override the default initial send window for these parameters.
    /// Gives an error on any value above 1000.
    ///
    /// You should probably not call this.
    pub fn set_initial_send_window(&mut self, v: u16) -> Result<()> {
        if v <= 1000 {
            self.initial_send_window = v;
            Ok(())
        } else {
            Err(Error::from(bad_api_usage!(
                "Tried to set an initial send window over 1000"
            )))
        }
    }

    /// Return the initial send window as set in this parameter set.
    pub fn initial_send_window(&self) -> u16 {
        self.initial_send_window
    }

    /// Override the default decision about whether to use ed25519
    /// identities in outgoing EXTEND2 cells.
    ///
    /// You should probably not call this.
    pub fn set_extend_by_ed25519_id(&mut self, v: bool) {
        self.extend_by_ed25519_id = v;
    }

    /// Return true if we're configured to extend by ed25519 ID; false
    /// otherwise.
    pub fn extend_by_ed25519_id(&self) -> bool {
        self.extend_by_ed25519_id
    }
}

/// A stream on a particular circuit.
#[derive(Clone, Debug)]
pub(crate) struct StreamTarget {
    /// Which hop of the circuit this stream is with.
    hop_num: HopNum,
    /// Reactor ID for this stream.
    stream_id: StreamId,
    /// Channel to send cells down.
    tx: mpsc::Sender<AnyRelayMsg>,
    /// Reference to the circuit that this stream is on.
    circ: Arc<ClientCirc>,
}

impl ClientCirc {
    /// Return a description of the first hop of this circuit.
    ///
    /// # Panics
    ///
    /// Panics if there is no first hop.  (This should be impossible outside of
    /// the tor-proto crate, but within the crate it's possible to have a
    /// circuit with no hops.)
    pub fn first_hop(&self) -> OwnedChanTarget {
        let first_hop = self
            .mutable
            .lock()
            .expect("poisoned lock")
            .path
            .first_hop()
            .expect("called first_hop on an un-constructed circuit");
        match first_hop {
            path::HopDetail::Relay(r) => r,
            #[cfg(feature = "hs-common")]
            path::HopDetail::Virtual => {
                panic!("somehow made a circuit with a virtual first hop.")
            }
        }
    }

    /// Return a description of all the hops in this circuit.
    ///
    /// This method is **deprecated** for several reasons:
    ///   * It performs a deep copy.
    ///   * It ignores virtual hops.
    ///   * It's not so extensible.
    ///
    /// Use [`ClientCirc::path_ref()`] instead.
    #[deprecated(since = "0.11.1", note = "Use path_ref() instead.")]
    pub fn path(&self) -> Vec<OwnedChanTarget> {
        #[allow(clippy::unnecessary_filter_map)] // clippy is blind to the cfg
        self.mutable
            .lock()
            .expect("poisoned lock")
            .path
            .all_hops()
            .into_iter()
            .filter_map(|hop| match hop {
                path::HopDetail::Relay(r) => Some(r),
                #[cfg(feature = "hs-common")]
                path::HopDetail::Virtual => None,
            })
            .collect()
    }

    /// Return a [`Path`] object describing all the hops in this circuit.
    ///
    /// Note that this `Path` is not automatically updated if the circuit is
    /// extended.
    pub fn path_ref(&self) -> Arc<Path> {
        self.mutable.lock().expect("poisoned_lock").path.clone()
    }

    /// Return a reference to the channel that this circuit is connected to.
    ///
    /// A client circuit is always connected to some relay via a [`Channel`].
    /// That relay has to be the same relay as the first hop in the client's
    /// path.
    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    /// Send a control message to the final hop on this circuit, and wait for
    /// one or more messages in reply.
    ///
    /// In detail:
    ///
    ///  1. `msg` is sent on the circuit.
    ///
    ///  2. After that, each message on the circuit that
    ///     isn't handled by the core machinery
    ///     is passed to `reply_handler`.
    ///     So any reply to `msg` will be seen by `reply_handler`.
    ///
    ///  3. When `reply_handler` returns `UninstallHandler`,
    ///     the handler is dropped, and then no more control messages
    ///     will be accepted.
    ///
    /// This function returns as soon as the message is sent
    /// and `reply_handler` is installed.
    /// `reply_handler` will probably want to have ownership of a
    /// an inter-task sending channel,
    /// so that it can communicate its results,
    /// and inform the caller when it is uninstalled.
    ///
    /// Note that it is quite possible to use this function to violate the tor
    /// protocol; most users of this API will not need to call it.  It is used
    /// to implement most of the onion service handshake.
    ///
    /// # Limitations
    ///
    /// Only one call to `send_control_message` may be active at any one time,
    /// for any one circuit.
    /// This generally means that this function should not be called
    /// on a circuit which might be shared with anyone else.
    ///
    /// Likewise, it is forbidden to try to extend the circuit,
    /// while `send_control_message` is running.
    ///
    /// If these restrictions are violated, the circuit will be closed with an error.
    ///
    /// After `send_control_message` has returned, the circuit may be extended,
    /// or `send_control_message` called again.
    /// Note that it is not currently possible, with this API,
    /// to send a followup control message,
    /// while continuously monitoring for relevant incoming cells:
    /// there will necessarily be a gap between `send_control_message` returning,
    /// and being called again.
    //
    // TODO hs (Diziet): IMO the above limitation ^ shows the API is not right.
    // There should probably be a separate function for when you're having
    // a conversation, that *doesn't* mess with the handler.
    // Something like this maybe:
    //     // This behaves like `send_control_message`, but the returned
    //     // future is a named type.
    //     //
    //     // Having two `Conversations` at once is forbidden.
    //     fn converse_directly(
    //         &self,
    //         msgs: impl Iterator<Item=AnyRelayCell>,
    //         reply_handler: impl MsgHandler + Send + 'static,
    //     ) -> Result<Conversation<'_>>;
    //     impl Future for Conversation<'_> {
    //         type Output = Result<()>; ....
    //     }
    //     impl Conversation {
    //         // `MsgHandler::msg_handler` gets a `Conversation<'_>` too
    //         // in case it wants to call this
    //         fn send_msg(&self, msg: AnyRelayCell> -> Result<()> {...}
    //     }
    // Further discussion here:
    //    https://gitlab.torproject.org/tpo/core/arti/-/merge_requests/1218#note_2908120
    //
    // TODO hs: rename this. "control_messages" is kind of ambiguous; we use
    //   "control" for a lot of other things. We say "meta" elsewhere in the
    //   reactor code, but "meta messages" just sounds odd.
    //
    // TODO hs: it might be nice to avoid exposing tor-cell APIs in the
    //   tor-proto interface.
    //
    // TODO hs: I'm not sure this API is the right shape...
    //
    // It's a little overkill for ESTABLISH_RENDEZVOUS where we expect a single
    // RENDEZVOUS_ESTABLISHED, then eventually a single RENDEZVOUS2. It's also a
    // little overkill for INTRODUCE1 where we expect an INTRODUCE_ACK.
    //
    // It will work for it's good for ESTABLISH_INTRO where we expect an
    // INTRO_ESTABLISHED followed by a large number of INTRODUCE2-- though we
    // might regret an unbounded circuit?
    //
    // It isn't quite right for RENDEZVOUS1, where we expect no reply, and want
    // to send the message to the second-to-last hop (having added a virtual hop
    // to the circuit.)
    //
    // TODO hs: Possibly this function should use
    // HopNum or similar to indicate which hop we're talking to, rather than
    // just doing "the last hop".

    // TODO hs: Perhaps the stream here should yield a different type. Ian
    // thinks maybe we should store a callback instead.
    //
    #[cfg(feature = "send-control-msg")]
    pub async fn send_control_message(
        &self,
        msg: tor_cell::relaycell::msg::AnyRelayMsg,
        reply_handler: impl MsgHandler + Send + 'static,
    ) -> Result<()> {
        let msg = tor_cell::relaycell::AnyRelayCell::new(0.into(), msg);
        let last_hop = self
            .mutable
            .lock()
            .expect("poisoned lock")
            .path
            .last_hop_num()
            .ok_or_else(|| internal!("no last hop index"))?;
        let handler = Box::new(msghandler::UserMsgHandler::new(last_hop, reply_handler));
        let (sender, receiver) = oneshot::channel();

        let ctrl_msg = CtrlMsg::SendMsgAndInstallHandler {
            msg,
            handler,
            sender,
        };
        self.control
            .unbounded_send(ctrl_msg)
            .map_err(|_| Error::CircuitClosed)?;

        receiver.await.map_err(|_| Error::CircuitClosed)?
    }

    /// Tell this circuit to begin allowing the final hop of the circuit to try
    /// to create new Tor streams, and to return those pending requests in an
    /// asynchronous stream.
    ///
    /// Ordinarily, these requests are rejected.  
    ///
    /// There can only be one stream of this type created on a given circuit at
    /// a time. If a such a stream already exists, this method will return an
    /// error.
    ///
    /// (This function is not yet implemented; right now, it will always panic.)
    ///
    /// Only onion services (and eventually) exit relays should call this
    /// method.
    #[cfg(feature = "hs-service")]
    #[allow(clippy::missing_panics_doc, unused_variables)] // TODO hss remove
    pub fn allow_stream_requests(
        &self,
        allow_commands: &[tor_cell::relaycell::RelayCmd],
    ) -> Result<impl futures::Stream<Item = crate::stream::IncomingStream>> {
        if false {
            return Ok(futures::stream::empty()); // TODO hss remove; this is just here for type inference.
        }
        todo!() // TODO hss implement.
    }

    /// Extend the circuit via the ntor handshake to a new target last
    /// hop.
    pub async fn extend_ntor<Tg>(&self, target: &Tg, params: &CircParameters) -> Result<()>
    where
        Tg: CircTarget,
    {
        let key = NtorPublicKey {
            id: *target
                .rsa_identity()
                .ok_or(Error::MissingId(RelayIdType::Ed25519))?,
            pk: *target.ntor_onion_key(),
        };
        let mut linkspecs = target
            .linkspecs()
            .map_err(into_internal!("Could not encode linkspecs for extend_ntor"))?;
        if !params.extend_by_ed25519_id() {
            linkspecs.retain(|ls| ls.lstype() != LinkSpecType::ED25519ID);
        }

        let (tx, rx) = oneshot::channel();

        let peer_id = OwnedChanTarget::from_chan_target(target);
        self.control
            .unbounded_send(CtrlMsg::ExtendNtor {
                peer_id,
                public_key: key,
                linkspecs,
                params: params.clone(),
                done: tx,
            })
            .map_err(|_| Error::CircuitClosed)?;

        rx.await.map_err(|_| Error::CircuitClosed)??;

        Ok(())
    }

    /// Extend this circuit by a single, "virtual" hop.
    ///
    /// A virtual hop is one for which we do not add an actual network connection
    /// between separate hosts (such as Relays).  We only add a layer of
    /// cryptography.
    ///
    /// This is used to implement onion services: the client and the service
    /// both build a circuit to a single rendezvous point, and tell the
    /// rendezvous point to relay traffic between their two circuits.  Having
    /// completed a [`handshake`] out of band[^1], the parties each extend their
    /// circuits by a single "virtual" encryption hop that represents their
    /// shared cryptographic context.
    ///
    /// Once a circuit has been extended in this way, it is an error to try to
    /// extend it in any other way.
    ///
    /// [^1]: Technically, the handshake is only _mostly_ out of band: the
    ///     client sends their half of the handshake in an ` message, and the
    ///     service's response is inline in its `RENDEZVOUS2` message.
    //
    // TODO hs: let's try to enforce the "you can't extend a circuit again once
    // it has been extended this way" property.  We could do that with internal
    // state, or some kind of a type state pattern.
    //
    // TODO hs: possibly we should take a set of Protovers, and not just `Params`.
    #[cfg(feature = "hs-common")]
    pub async fn extend_virtual(
        &self,
        protocol: handshake::RelayProtocol,
        role: handshake::HandshakeRole,
        seed: impl handshake::KeyGenerator,
        params: CircParameters,
    ) -> Result<()> {
        let (outbound, inbound) = protocol.construct_layers(role, seed)?;

        let (tx, rx) = oneshot::channel();
        let message = CtrlMsg::ExtendVirtual {
            cell_crypto: (outbound, inbound),
            params,
            done: tx,
        };

        self.control
            .unbounded_send(message)
            .map_err(|_| Error::CircuitClosed)?;

        rx.await.map_err(|_| Error::CircuitClosed)?
    }

    /// Helper, used to begin a stream.
    ///
    /// This function allocates a stream ID, and sends the message
    /// (like a BEGIN or RESOLVE), but doesn't wait for a response.
    ///
    /// The caller will typically want to see the first cell in response,
    /// to see whether it is e.g. an END or a CONNECTED.
    async fn begin_stream_impl(
        self: &Arc<ClientCirc>,
        begin_msg: AnyRelayMsg,
        cmd_checker: AnyCmdChecker,
    ) -> Result<(StreamReader, StreamTarget)> {
        // TODO: Possibly this should take a hop, rather than just
        // assuming it's the last hop.

        let hop_num = self
            .mutable
            .lock()
            .expect("poisoned lock")
            .path
            .last_hop_num()
            .ok_or_else(|| Error::from(internal!("Can't begin a stream at the 0th hop")))?;

        let (sender, receiver) = mpsc::channel(STREAM_READER_BUFFER);
        let (tx, rx) = oneshot::channel();
        let (msg_tx, msg_rx) = mpsc::channel(CIRCUIT_BUFFER_SIZE);

        self.control
            .unbounded_send(CtrlMsg::BeginStream {
                hop_num,
                message: begin_msg,
                sender,
                rx: msg_rx,
                done: tx,
                cmd_checker,
            })
            .map_err(|_| Error::CircuitClosed)?;

        let stream_id = rx.await.map_err(|_| Error::CircuitClosed)??;

        let target = StreamTarget {
            circ: self.clone(),
            tx: msg_tx,
            hop_num,
            stream_id,
        };

        let reader = StreamReader {
            target: target.clone(),
            receiver,
            recv_window: StreamRecvWindow::new(RECV_WINDOW_INIT),
            ended: false,
        };

        Ok((reader, target))
    }

    /// Start a DataStream (anonymized connection) to the given
    /// address and port, using a BEGIN cell.
    async fn begin_data_stream(
        self: &Arc<ClientCirc>,
        msg: AnyRelayMsg,
        optimistic: bool,
    ) -> Result<DataStream> {
        let (reader, target) = self
            .begin_stream_impl(msg, DataCmdChecker::new_any())
            .await?;
        let mut stream = DataStream::new(reader, target);
        if !optimistic {
            stream.wait_for_connection().await?;
        }
        Ok(stream)
    }

    /// Start a stream to the given address and port, using a BEGIN
    /// cell.
    ///
    /// The use of a string for the address is intentional: you should let
    /// the remote Tor relay do the hostname lookup for you.
    pub async fn begin_stream(
        self: &Arc<ClientCirc>,
        target: &str,
        port: u16,
        parameters: Option<StreamParameters>,
    ) -> Result<DataStream> {
        let parameters = parameters.unwrap_or_default();
        let begin_flags = parameters.begin_flags();
        let optimistic = parameters.is_optimistic();
        let beginmsg = Begin::new(target, port, begin_flags)
            .map_err(|e| Error::from_cell_enc(e, "begin message"))?;
        self.begin_data_stream(beginmsg.into(), optimistic).await
    }

    /// Start a new stream to the last relay in the circuit, using
    /// a BEGIN_DIR cell.
    pub async fn begin_dir_stream(self: Arc<ClientCirc>) -> Result<DataStream> {
        // Note that we always open begindir connections optimistically.
        // Since they are local to a relay that we've already authenticated
        // with and built a circuit to, there should be no additional checks
        // we need to perform to see whether the BEGINDIR will succeed.
        self.begin_data_stream(AnyRelayMsg::BeginDir(Default::default()), true)
            .await
    }

    /// Perform a DNS lookup, using a RESOLVE cell with the last relay
    /// in this circuit.
    ///
    /// Note that this function does not check for timeouts; that's
    /// the caller's responsibility.
    pub async fn resolve(self: &Arc<ClientCirc>, hostname: &str) -> Result<Vec<IpAddr>> {
        let resolve_msg = Resolve::new(hostname);

        let resolved_msg = self.try_resolve(resolve_msg).await?;

        resolved_msg
            .into_answers()
            .into_iter()
            .filter_map(|(val, _)| match resolvedval_to_result(val) {
                Ok(ResolvedVal::Ip(ip)) => Some(Ok(ip)),
                Ok(_) => None,
                Err(e) => Some(Err(e)),
            })
            .collect()
    }

    /// Perform a reverse DNS lookup, by sending a RESOLVE cell with
    /// the last relay on this circuit.
    ///
    /// Note that this function does not check for timeouts; that's
    /// the caller's responsibility.
    pub async fn resolve_ptr(self: &Arc<ClientCirc>, addr: IpAddr) -> Result<Vec<String>> {
        let resolve_ptr_msg = Resolve::new_reverse(&addr);

        let resolved_msg = self.try_resolve(resolve_ptr_msg).await?;

        resolved_msg
            .into_answers()
            .into_iter()
            .filter_map(|(val, _)| match resolvedval_to_result(val) {
                Ok(ResolvedVal::Hostname(v)) => Some(
                    String::from_utf8(v)
                        .map_err(|_| Error::StreamProto("Resolved Hostname was not utf-8".into())),
                ),
                Ok(_) => None,
                Err(e) => Some(Err(e)),
            })
            .collect()
    }

    /// Helper: Send the resolve message, and read resolved message from
    /// resolve stream.
    async fn try_resolve(self: &Arc<ClientCirc>, msg: Resolve) -> Result<Resolved> {
        let (reader, _) = self
            .begin_stream_impl(msg.into(), ResolveCmdChecker::new_any())
            .await?;
        let mut resolve_stream = ResolveStream::new(reader);
        resolve_stream.read_msg().await
    }

    /// Shut down this circuit, along with all streams that are using it.
    /// Happens asynchronously (i.e. the circuit won't necessarily be done shutting down
    /// immediately after this function returns!).
    ///
    /// Note that other references to this circuit may exist.  If they
    /// do, they will stop working after you call this function.
    ///
    /// It's not necessary to call this method if you're just done
    /// with a circuit: the channel should close on its own once nothing
    /// is using it any more.
    pub fn terminate(&self) {
        let _ = self.control.unbounded_send(CtrlMsg::Shutdown);
    }

    /// Called when a circuit-level protocol error has occurred and the
    /// circuit needs to shut down.
    ///
    /// This is a separate function because we may eventually want to have
    /// it do more than just shut down.
    ///
    /// As with `terminate`, this function is asynchronous.
    pub(crate) fn protocol_error(&self) {
        self.terminate();
    }

    /// Return true if this circuit is closed and therefore unusable.
    pub fn is_closing(&self) -> bool {
        self.control.is_closed()
    }

    /// Return a process-unique identifier for this circuit.
    pub fn unique_id(&self) -> UniqId {
        self.unique_id
    }

    /// Return the number of hops in this circuit.
    ///
    /// NOTE: This function will currently return only the number of hops
    /// _currently_ in the circuit. If there is an extend operation in progress,
    /// the currently pending hop may or may not be counted, depending on whether
    /// the extend operation finishes before this call is done.
    pub fn n_hops(&self) -> usize {
        self.mutable.lock().expect("poisoned lock").path.n_hops()
    }
}

impl PendingClientCirc {
    /// Instantiate a new circuit object: used from Channel::new_circ().
    ///
    /// Does not send a CREATE* cell on its own.
    ///
    ///
    pub(crate) fn new(
        id: CircId,
        channel: Channel,
        createdreceiver: oneshot::Receiver<CreateResponse>,
        input: mpsc::Receiver<ClientCircChanMsg>,
        unique_id: UniqId,
    ) -> (PendingClientCirc, reactor::Reactor) {
        let (reactor, control_tx, mutable) = Reactor::new(channel.clone(), id, unique_id, input);

        let circuit = ClientCirc {
            mutable,
            unique_id,
            control: control_tx,
            channel,
            #[cfg(test)]
            circid: id,
        };

        let pending = PendingClientCirc {
            recvcreated: createdreceiver,
            circ: Arc::new(circuit),
        };
        (pending, reactor)
    }

    /// Testing only: Extract the circuit ID for this pending circuit.
    #[cfg(test)]
    pub(crate) fn peek_circid(&self) -> CircId {
        self.circ.circid
    }

    /// Use the (questionable!) CREATE_FAST handshake to connect to the
    /// first hop of this circuit.
    ///
    /// There's no authentication in CRATE_FAST,
    /// so we don't need to know whom we're connecting to: we're just
    /// connecting to whichever relay the channel is for.
    pub async fn create_firsthop_fast(self, params: &CircParameters) -> Result<Arc<ClientCirc>> {
        let (tx, rx) = oneshot::channel();
        self.circ
            .control
            .unbounded_send(CtrlMsg::Create {
                recv_created: self.recvcreated,
                handshake: CircuitHandshake::CreateFast,
                params: params.clone(),
                done: tx,
            })
            .map_err(|_| Error::CircuitClosed)?;

        rx.await.map_err(|_| Error::CircuitClosed)??;

        Ok(self.circ)
    }

    /// Use the ntor handshake to connect to the first hop of this circuit.
    ///
    /// Note that the provided 'target' must match the channel's target,
    /// or the handshake will fail.
    pub async fn create_firsthop_ntor<Tg>(
        self,
        target: &Tg,
        params: CircParameters,
    ) -> Result<Arc<ClientCirc>>
    where
        Tg: tor_linkspec::CircTarget,
    {
        let (tx, rx) = oneshot::channel();

        self.circ
            .control
            .unbounded_send(CtrlMsg::Create {
                recv_created: self.recvcreated,
                handshake: CircuitHandshake::Ntor {
                    public_key: NtorPublicKey {
                        id: *target
                            .rsa_identity()
                            .ok_or(Error::MissingId(RelayIdType::Rsa))?,
                        pk: *target.ntor_onion_key(),
                    },
                    ed_identity: *target
                        .ed_identity()
                        .ok_or(Error::MissingId(RelayIdType::Ed25519))?,
                },
                params: params.clone(),
                done: tx,
            })
            .map_err(|_| Error::CircuitClosed)?;

        rx.await.map_err(|_| Error::CircuitClosed)??;

        Ok(self.circ)
    }
}

/// An object that can put a given handshake into a ChanMsg for a CREATE*
/// cell, and unwrap a CREATED* cell.
trait CreateHandshakeWrap {
    /// Construct an appropriate ChanMsg to hold this kind of handshake.
    fn to_chanmsg(&self, bytes: Vec<u8>) -> AnyChanMsg;
    /// Decode a ChanMsg to an appropriate handshake value, checking
    /// its type.
    fn decode_chanmsg(&self, msg: CreateResponse) -> Result<Vec<u8>>;
}

/// A CreateHandshakeWrap that generates CREATE_FAST and handles CREATED_FAST.
struct CreateFastWrap;

impl CreateHandshakeWrap for CreateFastWrap {
    fn to_chanmsg(&self, bytes: Vec<u8>) -> AnyChanMsg {
        chancell::msg::CreateFast::new(bytes).into()
    }
    fn decode_chanmsg(&self, msg: CreateResponse) -> Result<Vec<u8>> {
        use CreateResponse::*;
        match msg {
            CreatedFast(m) => Ok(m.into_handshake()),
            Destroy(_) => Err(Error::CircRefused(
                "Relay replied to CREATE_FAST with DESTROY.",
            )),
            _ => Err(Error::CircProto(format!(
                "Relay replied to CREATE_FAST with unexpected cell: {}",
                msg
            ))),
        }
    }
}

/// A CreateHandshakeWrap that generates CREATE2 and handles CREATED2
struct Create2Wrap {
    /// The handshake type to put in the CREATE2 cell.
    handshake_type: u16,
}
impl CreateHandshakeWrap for Create2Wrap {
    fn to_chanmsg(&self, bytes: Vec<u8>) -> AnyChanMsg {
        chancell::msg::Create2::new(self.handshake_type, bytes).into()
    }
    fn decode_chanmsg(&self, msg: CreateResponse) -> Result<Vec<u8>> {
        use CreateResponse::*;
        match msg {
            Created2(m) => Ok(m.into_body()),
            Destroy(_) => Err(Error::CircRefused("Relay replied to CREATE2 with DESTROY.")),
            _ => Err(Error::CircProto(format!(
                "Relay replied to CREATE2 with unexpected cell {}",
                msg
            ))),
        }
    }
}

impl StreamTarget {
    /// Deliver a relay message for the stream that owns this StreamTarget.
    ///
    /// The StreamTarget will set the correct stream ID and pick the
    /// right hop, but will not validate that the message is well-formed
    /// or meaningful in context.
    pub(crate) async fn send(&mut self, msg: AnyRelayMsg) -> Result<()> {
        self.tx.send(msg).await.map_err(|_| Error::CircuitClosed)?;
        Ok(())
    }

    /// Called when a circuit-level protocol error has occurred and the
    /// circuit needs to shut down.
    pub(crate) fn protocol_error(&mut self) {
        self.circ.protocol_error();
    }

    /// Send a SENDME cell for this stream.
    pub(crate) fn send_sendme(&mut self) -> Result<()> {
        self.circ
            .control
            .unbounded_send(CtrlMsg::SendSendme {
                stream_id: self.stream_id,
                hop_num: self.hop_num,
            })
            .map_err(|_| Error::CircuitClosed)?;
        Ok(())
    }

    /// Return a reference to the circuit that this `StreamTarget` is using.
    #[cfg(feature = "experimental-api")]
    pub(crate) fn circuit(&self) -> &Arc<ClientCirc> {
        &self.circ
    }
}

/// Convert a [`ResolvedVal`] into a Result, based on whether or not
/// it represents an error.
fn resolvedval_to_result(val: ResolvedVal) -> Result<ResolvedVal> {
    match val {
        ResolvedVal::TransientError => Err(Error::ResolveError(ResolveError::Transient)),
        ResolvedVal::NontransientError => Err(Error::ResolveError(ResolveError::Nontransient)),
        ResolvedVal::Unrecognized(_, _) => Err(Error::ResolveError(ResolveError::Unrecognized)),
        _ => Ok(val),
    }
}

#[cfg(test)]
mod test {
    // @@ begin test lint list maintained by maint/add_warning @@
    #![allow(clippy::bool_assert_comparison)]
    #![allow(clippy::clone_on_copy)]
    #![allow(clippy::dbg_macro)]
    #![allow(clippy::print_stderr)]
    #![allow(clippy::print_stdout)]
    #![allow(clippy::single_char_pattern)]
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::unchecked_duration_subtraction)]
    //! <!-- @@ end test lint list maintained by maint/add_warning @@ -->

    use super::*;
    use crate::channel::OpenChanCellS2C;
    use crate::channel::{test::new_reactor, CodecError};
    use crate::crypto::cell::RelayCellBody;
    use chanmsg::{AnyChanMsg, Created2, CreatedFast};
    use futures::channel::mpsc::{Receiver, Sender};
    use futures::io::{AsyncReadExt, AsyncWriteExt};
    use futures::sink::SinkExt;
    use futures::stream::StreamExt;
    use futures::task::SpawnExt;
    use hex_literal::hex;
    use std::time::Duration;
    use tor_basic_utils::test_rng::testing_rng;
    use tor_cell::chancell::{msg as chanmsg, AnyChanCell, BoxedCellBody};
    use tor_cell::relaycell::{msg as relaymsg, AnyRelayCell, StreamId};
    use tor_linkspec::OwnedCircTarget;
    use tor_rtcompat::{Runtime, SleepProvider};
    use tracing::trace;

    fn rmsg_to_ccmsg<ID>(id: ID, msg: relaymsg::AnyRelayMsg) -> ClientCircChanMsg
    where
        ID: Into<StreamId>,
    {
        let body: BoxedCellBody = AnyRelayCell::new(id.into(), msg)
            .encode(&mut testing_rng())
            .unwrap();
        let chanmsg = chanmsg::Relay::from(body);
        ClientCircChanMsg::Relay(chanmsg)
    }

    /// return an example OwnedCircTarget that can get used for an ntor handshake.
    fn example_target() -> OwnedCircTarget {
        let mut builder = OwnedCircTarget::builder();
        builder
            .chan_target()
            .ed_identity([6; 32].into())
            .rsa_identity([10; 20].into());
        builder
            .ntor_onion_key(
                hex!("395cb26b83b3cd4b91dba9913e562ae87d21ecdd56843da7ca939a6a69001253").into(),
            )
            .protocols("FlowCtrl=1".parse().unwrap())
            .build()
            .unwrap()
    }
    fn example_ntor_key() -> crate::crypto::handshake::ntor::NtorSecretKey {
        crate::crypto::handshake::ntor::NtorSecretKey::new(
            hex!("7789d92a89711a7e2874c61ea495452cfd48627b3ca2ea9546aafa5bf7b55803").into(),
            hex!("395cb26b83b3cd4b91dba9913e562ae87d21ecdd56843da7ca939a6a69001253").into(),
            [10_u8; 20].into(),
        )
    }

    fn working_fake_channel<R: Runtime>(
        rt: &R,
    ) -> (
        Channel,
        Receiver<AnyChanCell>,
        Sender<std::result::Result<OpenChanCellS2C, CodecError>>,
    ) {
        let (channel, chan_reactor, rx, tx) = new_reactor(rt.clone());
        rt.spawn(async {
            let _ignore = chan_reactor.run().await;
        })
        .unwrap();
        (channel, rx, tx)
    }

    async fn test_create<R: Runtime>(rt: &R, fast: bool) {
        // We want to try progressing from a pending circuit to a circuit
        // via a crate_fast handshake.

        use crate::crypto::handshake::{fast::CreateFastServer, ntor::NtorServer, ServerHandshake};

        let (chan, mut rx, _sink) = working_fake_channel(rt);
        let circid = 128.into();
        let (created_send, created_recv) = oneshot::channel();
        let (_circmsg_send, circmsg_recv) = mpsc::channel(64);
        let unique_id = UniqId::new(23, 17);

        let (pending, reactor) =
            PendingClientCirc::new(circid, chan, created_recv, circmsg_recv, unique_id);

        rt.spawn(async {
            let _ignore = reactor.run().await;
        })
        .unwrap();

        // Future to pretend to be a relay on the other end of the circuit.
        let simulate_relay_fut = async move {
            let mut rng = testing_rng();
            let create_cell = rx.next().await.unwrap();
            assert_eq!(create_cell.circid(), 128.into());
            let reply = if fast {
                let cf = match create_cell.msg() {
                    AnyChanMsg::CreateFast(cf) => cf,
                    _ => panic!(),
                };
                let (_, rep) = CreateFastServer::server(&mut rng, &[()], cf.handshake()).unwrap();
                CreateResponse::CreatedFast(CreatedFast::new(rep))
            } else {
                let c2 = match create_cell.msg() {
                    AnyChanMsg::Create2(c2) => c2,
                    _ => panic!(),
                };
                let (_, rep) =
                    NtorServer::server(&mut rng, &[example_ntor_key()], c2.body()).unwrap();
                CreateResponse::Created2(Created2::new(rep))
            };
            created_send.send(reply).unwrap();
        };
        // Future to pretend to be a client.
        let client_fut = async move {
            let target = example_target();
            let params = CircParameters::default();
            let ret = if fast {
                trace!("doing fast create");
                pending.create_firsthop_fast(&params).await
            } else {
                trace!("doing ntor create");
                pending.create_firsthop_ntor(&target, params).await
            };
            trace!("create done: result {:?}", ret);
            ret
        };

        let (circ, _) = futures::join!(client_fut, simulate_relay_fut);

        let _circ = circ.unwrap();

        // pfew!  We've build a circuit!  Let's make sure it has one hop.
        /* TODO: reinstate this.
        let inner = Arc::get_mut(&mut circuit).unwrap().c.into_inner();
        assert_eq!(inner.hops.len(), 1);
         */
    }

    #[test]
    fn test_create_fast() {
        tor_rtcompat::test_with_all_runtimes!(|rt| async move {
            test_create(&rt, true).await;
        });
    }
    #[test]
    fn test_create_ntor() {
        tor_rtcompat::test_with_all_runtimes!(|rt| async move {
            test_create(&rt, false).await;
        });
    }

    // An encryption layer that doesn't do any crypto.   Can be used
    // as inbound or outbound, but not both at once.
    pub(crate) struct DummyCrypto {
        counter_tag: [u8; 20],
        counter: u32,
        lasthop: bool,
    }
    impl DummyCrypto {
        fn next_tag(&mut self) -> &[u8; 20] {
            #![allow(clippy::identity_op)]
            self.counter_tag[0] = ((self.counter >> 0) & 255) as u8;
            self.counter_tag[1] = ((self.counter >> 8) & 255) as u8;
            self.counter_tag[2] = ((self.counter >> 16) & 255) as u8;
            self.counter_tag[3] = ((self.counter >> 24) & 255) as u8;
            self.counter += 1;
            &self.counter_tag
        }
    }

    impl crate::crypto::cell::OutboundClientLayer for DummyCrypto {
        fn originate_for(&mut self, _cell: &mut RelayCellBody) -> &[u8] {
            self.next_tag()
        }
        fn encrypt_outbound(&mut self, _cell: &mut RelayCellBody) {}
    }
    impl crate::crypto::cell::InboundClientLayer for DummyCrypto {
        fn decrypt_inbound(&mut self, _cell: &mut RelayCellBody) -> Option<&[u8]> {
            if self.lasthop {
                Some(self.next_tag())
            } else {
                None
            }
        }
    }
    impl DummyCrypto {
        pub(crate) fn new(lasthop: bool) -> Self {
            DummyCrypto {
                counter_tag: [0; 20],
                counter: 0,
                lasthop,
            }
        }
    }

    // Helper: set up a 3-hop circuit with no encryption, where the
    // next inbound message seems to come from hop next_msg_from
    async fn newcirc_ext<R: Runtime>(
        rt: &R,
        chan: Channel,
        next_msg_from: HopNum,
    ) -> (Arc<ClientCirc>, mpsc::Sender<ClientCircChanMsg>) {
        let circid = 128.into();
        let (_created_send, created_recv) = oneshot::channel();
        let (circmsg_send, circmsg_recv) = mpsc::channel(64);
        let unique_id = UniqId::new(23, 17);

        let (pending, reactor) =
            PendingClientCirc::new(circid, chan, created_recv, circmsg_recv, unique_id);

        rt.spawn(async {
            let _ignore = reactor.run().await;
        })
        .unwrap();

        let PendingClientCirc {
            circ,
            recvcreated: _,
        } = pending;

        for idx in 0_u8..3 {
            let params = CircParameters::default();
            let (tx, rx) = oneshot::channel();
            circ.control
                .unbounded_send(CtrlMsg::AddFakeHop {
                    fwd_lasthop: idx == 2,
                    rev_lasthop: idx == u8::from(next_msg_from),
                    params,
                    done: tx,
                })
                .unwrap();
            rx.await.unwrap().unwrap();
        }

        (circ, circmsg_send)
    }

    // Helper: set up a 3-hop circuit with no encryption, where the
    // next inbound message seems to come from hop next_msg_from
    async fn newcirc<R: Runtime>(
        rt: &R,
        chan: Channel,
    ) -> (Arc<ClientCirc>, mpsc::Sender<ClientCircChanMsg>) {
        newcirc_ext(rt, chan, 2.into()).await
    }

    // Try sending a cell via send_relay_cell
    #[test]
    fn send_simple() {
        tor_rtcompat::test_with_all_runtimes!(|rt| async move {
            let (chan, mut rx, _sink) = working_fake_channel(&rt);
            let (circ, _send) = newcirc(&rt, chan).await;
            let begindir = AnyRelayCell::new(0.into(), AnyRelayMsg::BeginDir(Default::default()));
            circ.control
                .unbounded_send(CtrlMsg::SendRelayCell {
                    hop: 2.into(),
                    early: false,
                    cell: begindir,
                })
                .unwrap();

            // Here's what we tried to put on the TLS channel.  Note that
            // we're using dummy relay crypto for testing convenience.
            let rcvd = rx.next().await.unwrap();
            assert_eq!(rcvd.circid(), 128.into());
            let m = match rcvd.into_circid_and_msg().1 {
                AnyChanMsg::Relay(r) => AnyRelayCell::decode(r.into_relay_body()).unwrap(),
                _ => panic!(),
            };
            assert!(matches!(m.msg(), AnyRelayMsg::BeginDir(_)));
        });
    }

    // NOTE(eta): this test is commented out because it basically tested implementation details
    //            of the old code which are hard to port to the reactor version, and the behaviour
    //            is covered by the extend tests anyway, so I don't think it's worth it.

    /*
    // Try getting a "meta-cell", which is what we're calling those not
    // for a specific circuit.
    #[async_test]
    async fn recv_meta() {
        let (chan, _, _sink) = working_fake_channel();
        let (circ, mut reactor, mut sink) = newcirc(chan).await;

        // 1: Try doing it via handle_meta_cell directly.
        let meta_receiver = circ.register_meta_handler(2.into()).await.unwrap();
        let extended: RelayMsg = relaymsg::Extended2::new((*b"123").into()).into();
        {
            circ.c
                .lock()
                .await
                .handle_meta_cell(2.into(), extended.clone())
                .await
                .unwrap();
        }
        let msg = meta_receiver.await.unwrap().unwrap();
        assert!(matches!(msg, RelayMsg::Extended2(_)));

        // 2: Try doing it via the reactor.
        let meta_receiver = circ.register_meta_handler(2.into()).await.unwrap();
        sink.send(rmsg_to_ccmsg(0, extended.clone())).await.unwrap();
        reactor.run_once().await.unwrap();
        let msg = meta_receiver.await.unwrap().unwrap();
        assert!(matches!(msg, RelayMsg::Extended2(_)));

        // 3: Try getting a meta cell that we didn't want.
        let e = {
            circ.c
                .lock()
                .await
                .handle_meta_cell(2.into(), extended.clone())
                .await
                .err()
                .unwrap()
        };
        assert_eq!(
            format!("{}", e),
            "circuit protocol violation: Unexpected EXTENDED2 cell on client circuit"
        );

        // 3: Try getting a meta from a hop that we didn't want.
        let _receiver = circ.register_meta_handler(2.into()).await.unwrap();
        let e = {
            circ.c
                .lock()
                .await
                .handle_meta_cell(1.into(), extended.clone())
                .await
                .err()
                .unwrap()
        };
        assert_eq!(
            format!("{}", e),
            "circuit protocol violation: Unexpected EXTENDED2 cell from hop 1 on client circuit"
        );
    }
     */

    #[test]
    fn extend() {
        tor_rtcompat::test_with_all_runtimes!(|rt| async move {
            use crate::crypto::handshake::{ntor::NtorServer, ServerHandshake};

            let (chan, mut rx, _sink) = working_fake_channel(&rt);
            let (circ, mut sink) = newcirc(&rt, chan).await;
            let params = CircParameters::default();

            let extend_fut = async move {
                let target = example_target();
                circ.extend_ntor(&target, &params).await.unwrap();
                circ // gotta keep the circ alive, or the reactor would exit.
            };
            let reply_fut = async move {
                // We've disabled encryption on this circuit, so we can just
                // read the extend2 cell.
                let (id, chmsg) = rx.next().await.unwrap().into_circid_and_msg();
                assert_eq!(id, 128.into());
                let rmsg = match chmsg {
                    AnyChanMsg::RelayEarly(r) => AnyRelayCell::decode(r.into_relay_body()).unwrap(),
                    _ => panic!(),
                };
                let e2 = match rmsg.msg() {
                    AnyRelayMsg::Extend2(e2) => e2,
                    _ => panic!(),
                };
                let mut rng = testing_rng();
                let (_, reply) =
                    NtorServer::server(&mut rng, &[example_ntor_key()], e2.handshake()).unwrap();
                let extended2 = relaymsg::Extended2::new(reply).into();
                sink.send(rmsg_to_ccmsg(0, extended2)).await.unwrap();
                sink // gotta keep the sink alive, or the reactor will exit.
            };

            let (circ, _) = futures::join!(extend_fut, reply_fut);

            // Did we really add another hop?
            assert_eq!(circ.n_hops(), 4);

            // Do the path accessors report a reasonable outcome?
            #[allow(deprecated)]
            {
                let path = circ.path();
                assert_eq!(path.len(), 4);
                use tor_linkspec::HasRelayIds;
                assert_eq!(path[3].ed_identity(), example_target().ed_identity());
                assert_ne!(path[0].ed_identity(), example_target().ed_identity());
            }
            {
                let path = circ.path_ref();
                assert_eq!(path.n_hops(), 4);
                use tor_linkspec::HasRelayIds;
                assert_eq!(
                    path.hops()[3].as_chan_target().unwrap().ed_identity(),
                    example_target().ed_identity()
                );
                assert_ne!(
                    path.hops()[0].as_chan_target().unwrap().ed_identity(),
                    example_target().ed_identity()
                );
            }
        });
    }

    async fn bad_extend_test_impl<R: Runtime>(
        rt: &R,
        reply_hop: HopNum,
        bad_reply: ClientCircChanMsg,
    ) -> Error {
        let (chan, _rx, _sink) = working_fake_channel(rt);
        let (circ, mut sink) = newcirc_ext(rt, chan, reply_hop).await;
        let params = CircParameters::default();

        let target = example_target();
        #[allow(clippy::clone_on_copy)]
        let rtc = rt.clone();
        let sink_handle = rt
            .spawn_with_handle(async move {
                rtc.sleep(Duration::from_millis(100)).await;
                sink.send(bad_reply).await.unwrap();
                sink
            })
            .unwrap();
        let outcome = circ.extend_ntor(&target, &params).await;
        let _sink = sink_handle.await;

        assert_eq!(circ.n_hops(), 3);
        assert!(outcome.is_err());
        outcome.unwrap_err()
    }

    #[test]
    fn bad_extend_wronghop() {
        tor_rtcompat::test_with_all_runtimes!(|rt| async move {
            let extended2 = relaymsg::Extended2::new(vec![]).into();
            let cc = rmsg_to_ccmsg(0, extended2);

            let error = bad_extend_test_impl(&rt, 1.into(), cc).await;
            // This case shows up as a CircDestroy, since a message sent
            // from the wrong hop won't even be delivered to the extend
            // code's meta-handler.  Instead the unexpected message will cause
            // the circuit to get torn down.
            match error {
                Error::CircuitClosed => {}
                x => panic!("got other error: {}", x),
            }
        });
    }

    #[test]
    fn bad_extend_wrongtype() {
        tor_rtcompat::test_with_all_runtimes!(|rt| async move {
            let extended = relaymsg::Extended::new(vec![7; 200]).into();
            let cc = rmsg_to_ccmsg(0, extended);

            let error = bad_extend_test_impl(&rt, 2.into(), cc).await;
            match error {
                Error::BytesErr {
                    err: tor_bytes::Error::InvalidMessage(_),
                    object: "extended2 message",
                } => {}
                _ => panic!(),
            }
        });
    }

    #[test]
    fn bad_extend_destroy() {
        tor_rtcompat::test_with_all_runtimes!(|rt| async move {
            let cc = ClientCircChanMsg::Destroy(chanmsg::Destroy::new(4.into()));
            let error = bad_extend_test_impl(&rt, 2.into(), cc).await;
            match error {
                Error::CircuitClosed => {}
                _ => panic!(),
            }
        });
    }

    #[test]
    fn bad_extend_crypto() {
        tor_rtcompat::test_with_all_runtimes!(|rt| async move {
            let extended2 = relaymsg::Extended2::new(vec![99; 256]).into();
            let cc = rmsg_to_ccmsg(0, extended2);
            let error = bad_extend_test_impl(&rt, 2.into(), cc).await;
            assert!(matches!(error, Error::BadCircHandshakeAuth));
        });
    }

    #[test]
    fn begindir() {
        tor_rtcompat::test_with_all_runtimes!(|rt| async move {
            let (chan, mut rx, _sink) = working_fake_channel(&rt);
            let (circ, mut sink) = newcirc(&rt, chan).await;

            let begin_and_send_fut = async move {
                // Here we'll say we've got a circuit, and we want to
                // make a simple BEGINDIR request with it.
                let mut stream = circ.begin_dir_stream().await.unwrap();
                stream.write_all(b"HTTP/1.0 GET /\r\n").await.unwrap();
                stream.flush().await.unwrap();
                let mut buf = [0_u8; 1024];
                let n = stream.read(&mut buf).await.unwrap();
                assert_eq!(&buf[..n], b"HTTP/1.0 404 Not found\r\n");
                let n = stream.read(&mut buf).await.unwrap();
                assert_eq!(n, 0);
                stream
            };
            let reply_fut = async move {
                // We've disabled encryption on this circuit, so we can just
                // read the begindir cell.
                let (id, chmsg) = rx.next().await.unwrap().into_circid_and_msg();
                assert_eq!(id, 128.into()); // hardcoded circid.
                let rmsg = match chmsg {
                    AnyChanMsg::Relay(r) => AnyRelayCell::decode(r.into_relay_body()).unwrap(),
                    _ => panic!(),
                };
                let (streamid, rmsg) = rmsg.into_streamid_and_msg();
                assert!(matches!(rmsg, AnyRelayMsg::BeginDir(_)));

                // Reply with a Connected cell to indicate success.
                let connected = relaymsg::Connected::new_empty().into();
                sink.send(rmsg_to_ccmsg(streamid, connected)).await.unwrap();

                // Now read a DATA cell...
                let (id, chmsg) = rx.next().await.unwrap().into_circid_and_msg();
                assert_eq!(id, 128.into());
                let rmsg = match chmsg {
                    AnyChanMsg::Relay(r) => AnyRelayCell::decode(r.into_relay_body()).unwrap(),
                    _ => panic!(),
                };
                let (streamid_2, rmsg) = rmsg.into_streamid_and_msg();
                assert_eq!(streamid_2, streamid);
                if let AnyRelayMsg::Data(d) = rmsg {
                    assert_eq!(d.as_ref(), &b"HTTP/1.0 GET /\r\n"[..]);
                } else {
                    panic!();
                }

                // Write another data cell in reply!
                let data = relaymsg::Data::new(b"HTTP/1.0 404 Not found\r\n")
                    .unwrap()
                    .into();
                sink.send(rmsg_to_ccmsg(streamid, data)).await.unwrap();

                // Send an END cell to say that the conversation is over.
                let end = relaymsg::End::new_with_reason(relaymsg::EndReason::DONE).into();
                sink.send(rmsg_to_ccmsg(streamid, end)).await.unwrap();

                (rx, sink) // gotta keep these alive, or the reactor will exit.
            };

            let (_stream, (_rx, _sink)) = futures::join!(begin_and_send_fut, reply_fut);
        });
    }

    // Set up a circuit and stream that expects some incoming SENDMEs.
    async fn setup_incoming_sendme_case<R: Runtime>(
        rt: &R,
        n_to_send: usize,
    ) -> (
        Arc<ClientCirc>,
        DataStream,
        mpsc::Sender<ClientCircChanMsg>,
        StreamId,
        usize,
        Receiver<AnyChanCell>,
        Sender<std::result::Result<OpenChanCellS2C, CodecError>>,
    ) {
        let (chan, mut rx, sink2) = working_fake_channel(rt);
        let (circ, mut sink) = newcirc(rt, chan).await;

        let circ_clone = circ.clone();
        let begin_and_send_fut = async move {
            // Take our circuit and make a stream on it.
            let mut stream = circ_clone
                .begin_stream("www.example.com", 443, None)
                .await
                .unwrap();
            let junk = [0_u8; 1024];
            let mut remaining = n_to_send;
            while remaining > 0 {
                let n = std::cmp::min(remaining, junk.len());
                stream.write_all(&junk[..n]).await.unwrap();
                remaining -= n;
            }
            stream.flush().await.unwrap();
            stream
        };

        let receive_fut = async move {
            // Read the begindir cell.
            let (_id, chmsg) = rx.next().await.unwrap().into_circid_and_msg();
            let rmsg = match chmsg {
                AnyChanMsg::Relay(r) => AnyRelayCell::decode(r.into_relay_body()).unwrap(),
                _ => panic!(),
            };
            let (streamid, rmsg) = rmsg.into_streamid_and_msg();
            assert!(matches!(rmsg, AnyRelayMsg::Begin(_)));
            // Reply with a connected cell...
            let connected = relaymsg::Connected::new_empty().into();
            sink.send(rmsg_to_ccmsg(streamid, connected)).await.unwrap();
            // Now read bytes from the stream until we have them all.
            let mut bytes_received = 0_usize;
            let mut cells_received = 0_usize;
            while bytes_received < n_to_send {
                // Read a data cell, and remember how much we got.
                let (id, chmsg) = rx.next().await.unwrap().into_circid_and_msg();
                assert_eq!(id, 128.into());

                let rmsg = match chmsg {
                    AnyChanMsg::Relay(r) => AnyRelayCell::decode(r.into_relay_body()).unwrap(),
                    _ => panic!(),
                };
                let (streamid2, rmsg) = rmsg.into_streamid_and_msg();
                assert_eq!(streamid2, streamid);
                if let AnyRelayMsg::Data(dat) = rmsg {
                    cells_received += 1;
                    bytes_received += dat.as_ref().len();
                } else {
                    panic!();
                }
            }

            (sink, streamid, cells_received, rx)
        };

        let (stream, (sink, streamid, cells_received, rx)) =
            futures::join!(begin_and_send_fut, receive_fut);

        (circ, stream, sink, streamid, cells_received, rx, sink2)
    }

    #[test]
    fn accept_valid_sendme() {
        tor_rtcompat::test_with_all_runtimes!(|rt| async move {
            let (circ, _stream, mut sink, streamid, cells_received, _rx, _sink2) =
                setup_incoming_sendme_case(&rt, 300 * 498 + 3).await;

            assert_eq!(cells_received, 301);

            // Make sure that the circuit is indeed expecting the right sendmes
            {
                let (tx, rx) = oneshot::channel();
                circ.control
                    .unbounded_send(CtrlMsg::QuerySendWindow {
                        hop: 2.into(),
                        done: tx,
                    })
                    .unwrap();
                let (window, tags) = rx.await.unwrap().unwrap();
                assert_eq!(window, 1000 - 301);
                assert_eq!(tags.len(), 3);
                // 100
                assert_eq!(
                    tags[0],
                    sendme::CircTag::from(hex!("6400000000000000000000000000000000000000"))
                );
                // 200
                assert_eq!(
                    tags[1],
                    sendme::CircTag::from(hex!("c800000000000000000000000000000000000000"))
                );
                // 300
                assert_eq!(
                    tags[2],
                    sendme::CircTag::from(hex!("2c01000000000000000000000000000000000000"))
                );
            }

            let reply_with_sendme_fut = async move {
                // make and send a circuit-level sendme.
                let c_sendme =
                    relaymsg::Sendme::new_tag(hex!("6400000000000000000000000000000000000000"))
                        .into();
                sink.send(rmsg_to_ccmsg(0_u16, c_sendme)).await.unwrap();

                // Make and send a stream-level sendme.
                let s_sendme = relaymsg::Sendme::new_empty().into();
                sink.send(rmsg_to_ccmsg(streamid, s_sendme)).await.unwrap();

                sink
            };

            let _sink = reply_with_sendme_fut.await;

            // FIXME(eta): this is a hacky way of waiting for the reactor to run before doing the below
            //             query; should find some way to properly synchronize to avoid flakiness
            rt.sleep(Duration::from_millis(100)).await;
            // Now make sure that the circuit is still happy, and its
            // window is updated.
            {
                let (tx, rx) = oneshot::channel();
                circ.control
                    .unbounded_send(CtrlMsg::QuerySendWindow {
                        hop: 2.into(),
                        done: tx,
                    })
                    .unwrap();
                let (window, _tags) = rx.await.unwrap().unwrap();
                assert_eq!(window, 1000 - 201);
            }
        });
    }

    #[test]
    fn invalid_circ_sendme() {
        tor_rtcompat::test_with_all_runtimes!(|rt| async move {
            // Same setup as accept_valid_sendme() test above but try giving
            // a sendme with the wrong tag.

            let (circ, _stream, mut sink, _streamid, _cells_received, _rx, _sink2) =
                setup_incoming_sendme_case(&rt, 300 * 498 + 3).await;

            let reply_with_sendme_fut = async move {
                // make and send a circuit-level sendme with a bad tag.
                let c_sendme =
                    relaymsg::Sendme::new_tag(hex!("FFFF0000000000000000000000000000000000FF"))
                        .into();
                sink.send(rmsg_to_ccmsg(0_u16, c_sendme)).await.unwrap();
                sink
            };

            let _sink = reply_with_sendme_fut.await;

            let mut tries = 0;
            // FIXME(eta): we aren't testing the error message like we used to; however, we can at least
            //             check whether the reactor dies as a result of receiving invalid data.
            while !circ.control.is_closed() {
                // TODO: Don't sleep in tests.
                rt.sleep(Duration::from_millis(100)).await;
                tries += 1;
                if tries > 10 {
                    panic!("reactor continued running after invalid sendme");
                }
            }

            // TODO: check that the circuit is shut down too
        });
    }

    #[test]
    fn basic_params() {
        use super::CircParameters;
        let mut p = CircParameters::default();
        assert_eq!(p.initial_send_window(), 1000);
        assert!(p.extend_by_ed25519_id());

        assert!(p.set_initial_send_window(500).is_ok());
        p.set_extend_by_ed25519_id(false);
        assert_eq!(p.initial_send_window(), 500);
        assert!(!p.extend_by_ed25519_id());

        assert!(p.set_initial_send_window(9000).is_err());
        assert_eq!(p.initial_send_window(), 500);
    }
}
