//! Code related to tracking what activities a circuit can be used for.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::sync::Arc;
use std::time::SystemTime;
use tracing::debug;

use crate::path::{dirpath::DirPathBuilder, exitpath::ExitPathBuilder, TorPath};
use tor_chanmgr::ChannelUsage;
use tor_guardmgr::{GuardMgr, GuardMonitor, GuardUsable};
use tor_netdir::Relay;
use tor_netdoc::types::policy::PortPolicy;
use tor_rtcompat::Runtime;

use crate::isolation::{IsolationHelper, StreamIsolation};
use crate::mgr::{abstract_spec_find_supported, AbstractCirc, OpenEntry, RestrictionFailed};
use crate::Result;

/// An exit policy, as supported by the last hop of a circuit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExitPolicy {
    /// Permitted IPv4 ports.
    v4: Arc<PortPolicy>,
    /// Permitted IPv6 ports.
    v6: Arc<PortPolicy>,
}

/// A port that we want to connect to as a client.
///
/// Ordinarily, this is a TCP port, plus a flag to indicate whether we
/// must support IPv4 or IPv6.
#[derive(
    Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Default,
)]
pub struct TargetPort {
    /// True if this is a request to connect to an IPv6 address
    ipv6: bool,
    /// The port that the client wants to connect to
    port: u16,
}

impl TargetPort {
    /// Create a request to make sure that a circuit supports a given
    /// ipv4 exit port.
    pub fn ipv4(port: u16) -> TargetPort {
        TargetPort { ipv6: false, port }
    }

    /// Create a request to make sure that a circuit supports a given
    /// ipv6 exit port.
    pub fn ipv6(port: u16) -> TargetPort {
        TargetPort { ipv6: true, port }
    }

    /// Return true if this port is supported by the provided Relay.
    pub fn is_supported_by(&self, r: &tor_netdir::Relay<'_>) -> bool {
        if self.ipv6 {
            r.supports_exit_port_ipv6(self.port)
        } else {
            r.supports_exit_port_ipv4(self.port)
        }
    }
}

impl Display for TargetPort {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.port, if self.ipv6 { "v6" } else { "v4" })
    }
}

/// Set of requested target ports, mostly for use in error reporting
///
/// Displays nicely.
#[derive(Debug, Clone, Default)]
pub struct TargetPorts(Vec<TargetPort>);

impl From<&'_ [TargetPort]> for TargetPorts {
    fn from(ports: &'_ [TargetPort]) -> Self {
        TargetPorts(ports.into())
    }
}

impl Display for TargetPorts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let brackets = self.0.len() != 1;
        if brackets {
            write!(f, "[")?;
        }
        for (i, port) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{}", port)?;
        }
        if brackets {
            write!(f, "]")?;
        }
        Ok(())
    }
}

impl ExitPolicy {
    /// Make a new exit policy from a given Relay.
    pub(crate) fn from_relay(relay: &Relay<'_>) -> Self {
        Self {
            v4: relay.ipv4_policy(),
            v6: relay.ipv6_policy(),
        }
    }

    /// Return true if a given port is contained in this ExitPolicy.
    fn allows_port(&self, p: TargetPort) -> bool {
        let policy = if p.ipv6 { &self.v6 } else { &self.v4 };
        policy.allows_port(p.port)
    }

    /// Returns true if this policy allows any ports at all.
    fn allows_some_port(&self) -> bool {
        self.v4.allows_some_port() || self.v6.allows_some_port()
    }
}

/// The purpose for which a circuit is being created.
///
/// This type should stay internal to the circmgr crate for now: we'll probably
/// want to refactor it a lot.
#[derive(Clone, Debug)]
pub(crate) enum TargetCircUsage {
    /// Use for BEGINDIR-based non-anonymous directory connections
    Dir,
    /// Use to exit to one or more ports.
    Exit {
        /// List of ports the circuit has to allow.
        ///
        /// If this list of ports is empty, then the circuit doesn't need
        /// to support any particular port, but it still needs to be an exit.
        ports: Vec<TargetPort>,
        /// Isolation group the circuit shall be part of
        isolation: StreamIsolation,
    },
    /// For a circuit is only used for the purpose of building it.
    TimeoutTesting,
    /// For internal usage only: build a circuit preemptively, to reduce wait times.
    ///
    /// # Warning
    ///
    /// This **MUST NOT** be used by code outside of the preemptive circuit predictor. In
    /// particular, this usage doesn't support stream isolation, so using it to ask for
    /// circuits (for example, by passing it to `get_or_launch`) could be unsafe!
    Preemptive {
        /// A port the circuit has to allow, if specified.
        ///
        /// If this is `None`, we just want a circuit capable of doing DNS resolution.
        port: Option<TargetPort>,
        /// The number of exit circuits needed for a port
        circs: usize,
    },
}

/// The purposes for which a circuit is usable.
///
/// This type should stay internal to the circmgr crate for now: we'll probably
/// want to refactor it a lot.
#[derive(Clone, Debug)]
pub(crate) enum SupportedCircUsage {
    /// Usable for BEGINDIR-based non-anonymous directory connections
    Dir,
    /// Usable to exit to a set of ports.
    Exit {
        /// Exit policy of the circuit
        policy: ExitPolicy,
        /// Isolation group the circuit is part of. None when the circuit is not yet assigned to an
        /// isolation group.
        isolation: Option<StreamIsolation>,
    },
    /// This circuit is not suitable for any usage.
    NoUsage,
}

impl TargetCircUsage {
    /// Construct path for a given circuit purpose; return it and the
    /// usage that it _actually_ supports.
    pub(crate) fn build_path<'a, R: Rng, RT: Runtime>(
        &self,
        rng: &mut R,
        netdir: crate::DirInfo<'a>,
        guards: Option<&GuardMgr<RT>>,
        config: &crate::PathConfig,
        now: SystemTime,
    ) -> Result<(
        TorPath<'a>,
        SupportedCircUsage,
        Option<GuardMonitor>,
        Option<GuardUsable>,
    )> {
        match self {
            TargetCircUsage::Dir => {
                let (path, mon, usable) = DirPathBuilder::new().pick_path(rng, netdir, guards)?;
                Ok((path, SupportedCircUsage::Dir, mon, usable))
            }
            TargetCircUsage::Preemptive { port, .. } => {
                // FIXME(eta): this is copypasta from `TargetCircUsage::Exit`.
                let (path, mon, usable) = ExitPathBuilder::from_target_ports(port.iter().copied())
                    .pick_path(rng, netdir, guards, config, now)?;
                let policy = path
                    .exit_policy()
                    .expect("ExitPathBuilder gave us a one-hop circuit?");
                Ok((
                    path,
                    SupportedCircUsage::Exit {
                        policy,
                        isolation: None,
                    },
                    mon,
                    usable,
                ))
            }
            TargetCircUsage::Exit {
                ports: p,
                isolation,
            } => {
                let (path, mon, usable) = ExitPathBuilder::from_target_ports(p.clone())
                    .pick_path(rng, netdir, guards, config, now)?;
                let policy = path
                    .exit_policy()
                    .expect("ExitPathBuilder gave us a one-hop circuit?");
                Ok((
                    path,
                    SupportedCircUsage::Exit {
                        policy,
                        isolation: Some(isolation.clone()),
                    },
                    mon,
                    usable,
                ))
            }
            TargetCircUsage::TimeoutTesting => {
                let (path, mon, usable) = ExitPathBuilder::for_timeout_testing()
                    .pick_path(rng, netdir, guards, config, now)?;
                let policy = path.exit_policy();
                let usage = match policy {
                    Some(policy) if policy.allows_some_port() => SupportedCircUsage::Exit {
                        policy,
                        isolation: None,
                    },
                    _ => SupportedCircUsage::NoUsage,
                };

                Ok((path, usage, mon, usable))
            }
        }
    }
}

impl crate::mgr::AbstractSpec for SupportedCircUsage {
    type Usage = TargetCircUsage;

    fn supports(&self, target: &TargetCircUsage) -> bool {
        use SupportedCircUsage::*;
        match (self, target) {
            (Dir, TargetCircUsage::Dir) => true,
            (
                Exit {
                    policy: p1,
                    isolation: i1,
                },
                TargetCircUsage::Exit {
                    ports: p2,
                    isolation: i2,
                },
            ) => {
                i1.as_ref()
                    .map(|i1| i1.compatible_same_type(i2))
                    .unwrap_or(true)
                    && p2.iter().all(|port| p1.allows_port(*port))
            }
            (Exit { policy, isolation }, TargetCircUsage::Preemptive { port, .. }) => {
                if isolation.is_some() {
                    // If the circuit has a stream isolation, we might not be able to use it
                    // for new streams that don't share it.
                    return false;
                }
                if let Some(p) = port {
                    policy.allows_port(*p)
                } else {
                    true
                }
            }
            (Exit { .. } | NoUsage, TargetCircUsage::TimeoutTesting) => true,
            (_, _) => false,
        }
    }

    fn restrict_mut(
        &mut self,
        usage: &TargetCircUsage,
    ) -> std::result::Result<(), RestrictionFailed> {
        use SupportedCircUsage::*;

        match (self, usage) {
            (Dir, TargetCircUsage::Dir) => Ok(()),
            // This usage is only used to create circuits preemptively, and doesn't actually
            // correspond to any streams; accordingly, we don't need to modify the circuit's
            // acceptable usage at all.
            (Exit { .. }, TargetCircUsage::Preemptive { .. }) => Ok(()),
            (
                Exit {
                    isolation: ref mut isol1,
                    ..
                },
                TargetCircUsage::Exit { isolation: i2, .. },
            ) => {
                if let Some(i1) = isol1 {
                    if let Some(new_isolation) = i1.join_same_type(i2) {
                        // there was some isolation, and the requested usage is compatible, saving
                        // the new isolation into self
                        *isol1 = Some(new_isolation);
                        Ok(())
                    } else {
                        Err(RestrictionFailed::NotSupported)
                    }
                } else {
                    // there was no isolation yet on self, applying the restriction from usage
                    *isol1 = Some(i2.clone());
                    Ok(())
                }
            }
            (Exit { .. } | NoUsage, TargetCircUsage::TimeoutTesting) => Ok(()),
            (_, _) => Err(RestrictionFailed::NotSupported),
        }
    }

    fn find_supported<'a, 'b, C: AbstractCirc>(
        list: impl Iterator<Item = &'b mut OpenEntry<Self, C>>,
        usage: &TargetCircUsage,
    ) -> Vec<&'b mut OpenEntry<Self, C>> {
        match usage {
            TargetCircUsage::Preemptive { circs, .. } => {
                let supported = abstract_spec_find_supported(list, usage);
                // We need to have at least two circuits that support `port` in order
                // to reuse them; otherwise, we must create a new circuit, so
                // that we get closer to having two circuits.
                debug!(
                    "preemptive usage {:?} matches {} active circuits",
                    usage,
                    supported.len()
                );
                if supported.len() >= *circs {
                    supported
                } else {
                    vec![]
                }
            }
            _ => abstract_spec_find_supported(list, usage),
        }
    }

    fn channel_usage(&self) -> ChannelUsage {
        use ChannelUsage as CU;
        use SupportedCircUsage as SCU;
        match self {
            SCU::Dir => CU::Dir,
            SCU::Exit { .. } => CU::UserTraffic,
            SCU::NoUsage => CU::UselessCircuit,
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::isolation::test::{assert_isoleq, IsolationTokenEq};
    use crate::isolation::{IsolationToken, StreamIsolationBuilder};
    use crate::path::OwnedPath;
    use crate::test::OptDummyGuardMgr;
    use tor_basic_utils::test_rng::testing_rng;
    use tor_llcrypto::pk::ed25519::Ed25519Identity;
    use tor_netdir::testnet;

    impl IsolationTokenEq for TargetCircUsage {
        fn isol_eq(&self, other: &Self) -> bool {
            use TargetCircUsage::*;
            match (self, other) {
                (Dir, Dir) => true,
                (
                    Exit {
                        ports: p1,
                        isolation: is1,
                    },
                    Exit {
                        ports: p2,
                        isolation: is2,
                    },
                ) => p1 == p2 && is1.isol_eq(is2),
                (TimeoutTesting, TimeoutTesting) => true,
                (
                    Preemptive {
                        port: p1,
                        circs: c1,
                    },
                    Preemptive {
                        port: p2,
                        circs: c2,
                    },
                ) => p1 == p2 && c1 == c2,
                _ => false,
            }
        }
    }

    impl IsolationTokenEq for SupportedCircUsage {
        fn isol_eq(&self, other: &Self) -> bool {
            use SupportedCircUsage::*;
            match (self, other) {
                (Dir, Dir) => true,
                (
                    Exit {
                        policy: p1,
                        isolation: is1,
                    },
                    Exit {
                        policy: p2,
                        isolation: is2,
                    },
                ) => p1 == p2 && is1.isol_eq(is2),
                (NoUsage, NoUsage) => true,
                _ => false,
            }
        }
    }

    #[test]
    fn exit_policy() {
        use tor_netdir::testnet::construct_custom_netdir;
        use tor_netdoc::doc::netstatus::RelayFlags;

        let network = construct_custom_netdir(|idx, nb| {
            if (0x21..0x27).contains(&idx) {
                nb.rs.add_flags(RelayFlags::BAD_EXIT);
            }
        })
        .unwrap()
        .unwrap_if_sufficient()
        .unwrap();

        // Nodes with ID 0x0a through 0x13 and 0x1e through 0x27 are
        // exits.  Odd-numbered ones allow only ports 80 and 443;
        // even-numbered ones allow all ports.  Nodes with ID 0x21
        // through 0x27 are bad exits.
        let id_noexit: Ed25519Identity = [0x05; 32].into();
        let id_webexit: Ed25519Identity = [0x11; 32].into();
        let id_fullexit: Ed25519Identity = [0x20; 32].into();
        let id_badexit: Ed25519Identity = [0x25; 32].into();

        let not_exit = network.by_id(&id_noexit).unwrap();
        let web_exit = network.by_id(&id_webexit).unwrap();
        let full_exit = network.by_id(&id_fullexit).unwrap();
        let bad_exit = network.by_id(&id_badexit).unwrap();

        let ep_none = ExitPolicy::from_relay(&not_exit);
        let ep_web = ExitPolicy::from_relay(&web_exit);
        let ep_full = ExitPolicy::from_relay(&full_exit);
        let ep_bad = ExitPolicy::from_relay(&bad_exit);

        assert!(!ep_none.allows_port(TargetPort::ipv4(80)));
        assert!(!ep_none.allows_port(TargetPort::ipv4(9999)));

        assert!(ep_web.allows_port(TargetPort::ipv4(80)));
        assert!(ep_web.allows_port(TargetPort::ipv4(443)));
        assert!(!ep_web.allows_port(TargetPort::ipv4(9999)));

        assert!(ep_full.allows_port(TargetPort::ipv4(80)));
        assert!(ep_full.allows_port(TargetPort::ipv4(443)));
        assert!(ep_full.allows_port(TargetPort::ipv4(9999)));

        assert!(!ep_bad.allows_port(TargetPort::ipv4(80)));

        // Note that nobody in the testdir::network allows ipv6.
        assert!(!ep_none.allows_port(TargetPort::ipv6(80)));
        assert!(!ep_web.allows_port(TargetPort::ipv6(80)));
        assert!(!ep_full.allows_port(TargetPort::ipv6(80)));
        assert!(!ep_bad.allows_port(TargetPort::ipv6(80)));

        // Check is_supported_by while we're here.
        assert!(TargetPort::ipv4(80).is_supported_by(&web_exit));
        assert!(!TargetPort::ipv6(80).is_supported_by(&web_exit));
        assert!(!TargetPort::ipv6(80).is_supported_by(&bad_exit));
    }

    #[test]
    fn usage_ops() {
        use crate::mgr::AbstractSpec;
        // Make an exit-policy object that allows web on IPv4 and
        // smtp on IPv6.
        let policy = ExitPolicy {
            v4: Arc::new("accept 80,443".parse().unwrap()),
            v6: Arc::new("accept 23".parse().unwrap()),
        };
        let tok1 = IsolationToken::new();
        let tok2 = IsolationToken::new();
        let isolation = StreamIsolationBuilder::new()
            .owner_token(tok1)
            .build()
            .unwrap();
        let isolation2 = StreamIsolationBuilder::new()
            .owner_token(tok2)
            .build()
            .unwrap();

        let supp_dir = SupportedCircUsage::Dir;
        let targ_dir = TargetCircUsage::Dir;
        let supp_exit = SupportedCircUsage::Exit {
            policy: policy.clone(),
            isolation: Some(isolation.clone()),
        };
        let supp_exit_iso2 = SupportedCircUsage::Exit {
            policy: policy.clone(),
            isolation: Some(isolation2.clone()),
        };
        let supp_exit_no_iso = SupportedCircUsage::Exit {
            policy,
            isolation: None,
        };
        let supp_none = SupportedCircUsage::NoUsage;

        let targ_80_v4 = TargetCircUsage::Exit {
            ports: vec![TargetPort::ipv4(80)],
            isolation: isolation.clone(),
        };
        let targ_80_v4_iso2 = TargetCircUsage::Exit {
            ports: vec![TargetPort::ipv4(80)],
            isolation: isolation2,
        };
        let targ_80_23_v4 = TargetCircUsage::Exit {
            ports: vec![TargetPort::ipv4(80), TargetPort::ipv4(23)],
            isolation: isolation.clone(),
        };
        let targ_80_23_mixed = TargetCircUsage::Exit {
            ports: vec![TargetPort::ipv4(80), TargetPort::ipv6(23)],
            isolation: isolation.clone(),
        };
        let targ_999_v6 = TargetCircUsage::Exit {
            ports: vec![TargetPort::ipv6(999)],
            isolation,
        };
        let targ_testing = TargetCircUsage::TimeoutTesting;

        assert!(supp_dir.supports(&targ_dir));
        assert!(!supp_dir.supports(&targ_80_v4));
        assert!(!supp_exit.supports(&targ_dir));
        assert!(supp_exit.supports(&targ_80_v4));
        assert!(!supp_exit.supports(&targ_80_v4_iso2));
        assert!(supp_exit.supports(&targ_80_23_mixed));
        assert!(!supp_exit.supports(&targ_80_23_v4));
        assert!(!supp_exit.supports(&targ_999_v6));
        assert!(!supp_exit_iso2.supports(&targ_80_v4));
        assert!(supp_exit_iso2.supports(&targ_80_v4_iso2));
        assert!(supp_exit_no_iso.supports(&targ_80_v4));
        assert!(supp_exit_no_iso.supports(&targ_80_v4_iso2));
        assert!(!supp_exit_no_iso.supports(&targ_80_23_v4));
        assert!(!supp_none.supports(&targ_dir));
        assert!(!supp_none.supports(&targ_80_23_v4));
        assert!(!supp_none.supports(&targ_80_v4_iso2));
        assert!(!supp_dir.supports(&targ_testing));
        assert!(supp_exit.supports(&targ_testing));
        assert!(supp_exit_no_iso.supports(&targ_testing));
        assert!(supp_exit_iso2.supports(&targ_testing));
        assert!(supp_none.supports(&targ_testing));
    }

    #[test]
    fn restrict_mut() {
        use crate::mgr::AbstractSpec;

        let policy = ExitPolicy {
            v4: Arc::new("accept 80,443".parse().unwrap()),
            v6: Arc::new("accept 23".parse().unwrap()),
        };

        let tok1 = IsolationToken::new();
        let tok2 = IsolationToken::new();
        let isolation = StreamIsolationBuilder::new()
            .owner_token(tok1)
            .build()
            .unwrap();
        let isolation2 = StreamIsolationBuilder::new()
            .owner_token(tok2)
            .build()
            .unwrap();

        let supp_dir = SupportedCircUsage::Dir;
        let targ_dir = TargetCircUsage::Dir;
        let supp_exit = SupportedCircUsage::Exit {
            policy: policy.clone(),
            isolation: Some(isolation.clone()),
        };
        let supp_exit_iso2 = SupportedCircUsage::Exit {
            policy: policy.clone(),
            isolation: Some(isolation2.clone()),
        };
        let supp_exit_no_iso = SupportedCircUsage::Exit {
            policy,
            isolation: None,
        };
        let supp_none = SupportedCircUsage::NoUsage;
        let targ_exit = TargetCircUsage::Exit {
            ports: vec![TargetPort::ipv4(80)],
            isolation,
        };
        let targ_exit_iso2 = TargetCircUsage::Exit {
            ports: vec![TargetPort::ipv4(80)],
            isolation: isolation2,
        };
        let targ_testing = TargetCircUsage::TimeoutTesting;

        // not allowed, do nothing
        let mut supp_dir_c = supp_dir.clone();
        assert!(supp_dir_c.restrict_mut(&targ_exit).is_err());
        assert!(supp_dir_c.restrict_mut(&targ_testing).is_err());
        assert_isoleq!(supp_dir, supp_dir_c);

        let mut supp_exit_c = supp_exit.clone();
        assert!(supp_exit_c.restrict_mut(&targ_dir).is_err());
        assert_isoleq!(supp_exit, supp_exit_c);

        let mut supp_exit_c = supp_exit.clone();
        assert!(supp_exit_c.restrict_mut(&targ_exit_iso2).is_err());
        assert_isoleq!(supp_exit, supp_exit_c);

        let mut supp_exit_iso2_c = supp_exit_iso2.clone();
        assert!(supp_exit_iso2_c.restrict_mut(&targ_exit).is_err());
        assert_isoleq!(supp_exit_iso2, supp_exit_iso2_c);

        let mut supp_none_c = supp_none.clone();
        assert!(supp_none_c.restrict_mut(&targ_exit).is_err());
        assert!(supp_none_c.restrict_mut(&targ_dir).is_err());
        assert_isoleq!(supp_none_c, supp_none);

        // allowed but nothing to do
        let mut supp_dir_c = supp_dir.clone();
        supp_dir_c.restrict_mut(&targ_dir).unwrap();
        assert_isoleq!(supp_dir, supp_dir_c);

        let mut supp_exit_c = supp_exit.clone();
        supp_exit_c.restrict_mut(&targ_exit).unwrap();
        assert_isoleq!(supp_exit, supp_exit_c);

        let mut supp_exit_iso2_c = supp_exit_iso2.clone();
        supp_exit_iso2_c.restrict_mut(&targ_exit_iso2).unwrap();
        supp_none_c.restrict_mut(&targ_testing).unwrap();
        assert_isoleq!(supp_exit_iso2, supp_exit_iso2_c);

        let mut supp_none_c = supp_none.clone();
        supp_none_c.restrict_mut(&targ_testing).unwrap();
        assert_isoleq!(supp_none_c, supp_none);

        // allowed, do something
        let mut supp_exit_no_iso_c = supp_exit_no_iso.clone();
        supp_exit_no_iso_c.restrict_mut(&targ_exit).unwrap();
        assert!(supp_exit_no_iso_c.supports(&targ_exit));
        assert!(!supp_exit_no_iso_c.supports(&targ_exit_iso2));

        let mut supp_exit_no_iso_c = supp_exit_no_iso;
        supp_exit_no_iso_c.restrict_mut(&targ_exit_iso2).unwrap();
        assert!(!supp_exit_no_iso_c.supports(&targ_exit));
        assert!(supp_exit_no_iso_c.supports(&targ_exit_iso2));
    }

    #[test]
    fn buildpath() {
        use crate::mgr::AbstractSpec;
        let mut rng = testing_rng();
        let netdir = testnet::construct_netdir().unwrap_if_sufficient().unwrap();
        let di = (&netdir).into();
        let config = crate::PathConfig::default();
        let guards: OptDummyGuardMgr<'_> = None;
        let now = SystemTime::now();

        // Only doing basic tests for now.  We'll test the path
        // building code a lot more closely in the tests for TorPath
        // and friends.

        // First, a one-hop directory circuit
        let (p_dir, u_dir, _, _) = TargetCircUsage::Dir
            .build_path(&mut rng, di, guards, &config, now)
            .unwrap();
        assert!(matches!(u_dir, SupportedCircUsage::Dir));
        assert_eq!(p_dir.len(), 1);

        // Now an exit circuit, to port 995.
        let tok1 = IsolationToken::new();
        let isolation = StreamIsolationBuilder::new()
            .owner_token(tok1)
            .build()
            .unwrap();

        let exit_usage = TargetCircUsage::Exit {
            ports: vec![TargetPort::ipv4(995)],
            isolation: isolation.clone(),
        };
        let (p_exit, u_exit, _, _) = exit_usage
            .build_path(&mut rng, di, guards, &config, now)
            .unwrap();
        assert!(matches!(
            u_exit,
            SupportedCircUsage::Exit {
                isolation: ref iso,
                ..
            } if iso.isol_eq(&Some(isolation))
        ));
        assert!(u_exit.supports(&exit_usage));
        assert_eq!(p_exit.len(), 3);

        // Now try testing circuits.
        let (path, usage, _, _) = TargetCircUsage::TimeoutTesting
            .build_path(&mut rng, di, guards, &config, now)
            .unwrap();
        let path = match OwnedPath::try_from(&path).unwrap() {
            OwnedPath::ChannelOnly(_) => panic!("Impossible path type."),
            OwnedPath::Normal(p) => p,
        };
        assert_eq!(path.len(), 3);

        // Make sure that the usage is correct.
        let last_relay = netdir.by_ids(&path[2]).unwrap();
        let policy = ExitPolicy::from_relay(&last_relay);
        // We'll always get exits for these, since we try to build
        // paths with an exit if there are any exits.
        assert!(policy.allows_some_port());
        assert!(last_relay.policies_allow_some_port());
        assert_isoleq!(
            usage,
            SupportedCircUsage::Exit {
                policy,
                isolation: None
            }
        );
    }

    #[test]
    fn build_testing_noexit() {
        // Here we'll try to build paths for testing circuits on a network
        // with no exits.
        let mut rng = testing_rng();
        let netdir = testnet::construct_custom_netdir(|_idx, bld| {
            bld.md.parse_ipv4_policy("reject 1-65535").unwrap();
        })
        .unwrap()
        .unwrap_if_sufficient()
        .unwrap();
        let di = (&netdir).into();
        let config = crate::PathConfig::default();
        let guards: OptDummyGuardMgr<'_> = None;
        let now = SystemTime::now();

        let (path, usage, _, _) = TargetCircUsage::TimeoutTesting
            .build_path(&mut rng, di, guards, &config, now)
            .unwrap();
        assert_eq!(path.len(), 3);
        assert_isoleq!(usage, SupportedCircUsage::NoUsage);
    }

    #[test]
    fn display_target_ports() {
        let ports = [];
        assert_eq!(TargetPorts::from(&ports[..]).to_string(), "[]");

        let ports = [TargetPort::ipv4(80)];
        assert_eq!(TargetPorts::from(&ports[..]).to_string(), "80v4");
        let ports = [TargetPort::ipv4(80), TargetPort::ipv6(443)];
        assert_eq!(TargetPorts::from(&ports[..]).to_string(), "[80v4,443v6]");
    }
}
