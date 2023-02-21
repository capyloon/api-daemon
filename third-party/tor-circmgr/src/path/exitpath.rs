//! Code for building paths to an exit relay.

use super::TorPath;
use crate::{DirInfo, Error, PathConfig, Result, TargetPort};
use rand::Rng;
use std::time::SystemTime;
use tor_basic_utils::iter::FilterCount;
use tor_error::{bad_api_usage, internal};
use tor_guardmgr::{GuardMgr, GuardMonitor, GuardUsable};
use tor_linkspec::RelayIdSet;
use tor_netdir::{NetDir, Relay, SubnetConfig, WeightRole};
use tor_rtcompat::Runtime;

/// Internal representation of PathBuilder.
enum ExitPathBuilderInner<'a> {
    /// Request a path that allows exit to the given `TargetPort]`s.
    WantsPorts(Vec<TargetPort>),

    /// Request a path that allows exit to _any_ port.
    AnyExit {
        /// If false, then we fall back to non-exit nodes if we can't find an
        /// exit.
        strict: bool,
    },

    /// Request a path that uses a given relay as exit node.
    ChosenExit(Relay<'a>),
}

/// A PathBuilder that builds a path to an exit relay supporting a given
/// set of ports.
pub struct ExitPathBuilder<'a> {
    /// The inner ExitPathBuilder state.
    inner: ExitPathBuilderInner<'a>,
}

impl<'a> ExitPathBuilder<'a> {
    /// Create a new builder that will try to get an exit relay
    /// containing all the ports in `ports`.
    ///
    /// If the list of ports is empty, tries to get any exit relay at all.
    pub fn from_target_ports(wantports: impl IntoIterator<Item = TargetPort>) -> Self {
        let ports: Vec<TargetPort> = wantports.into_iter().collect();
        if ports.is_empty() {
            return Self::for_any_exit();
        }
        Self {
            inner: ExitPathBuilderInner::WantsPorts(ports),
        }
    }

    /// Create a new builder that will try to build a path with the given exit
    /// relay as the last hop.
    pub fn from_chosen_exit(exit_relay: Relay<'a>) -> Self {
        Self {
            inner: ExitPathBuilderInner::ChosenExit(exit_relay),
        }
    }

    /// Create a new builder that will try to get any exit relay at all.
    pub fn for_any_exit() -> Self {
        Self {
            inner: ExitPathBuilderInner::AnyExit { strict: true },
        }
    }

    /// Create a new builder that will try to get an exit relay, but which
    /// will be satisfied with a non-exit relay.
    pub(crate) fn for_timeout_testing() -> Self {
        Self {
            inner: ExitPathBuilderInner::AnyExit { strict: false },
        }
    }

    /// Find a suitable exit node from either the chosen exit or from the network directory.
    fn pick_exit<R: Rng>(
        &self,
        rng: &mut R,
        netdir: &'a NetDir,
        guard: Option<&Relay<'a>>,
        config: SubnetConfig,
    ) -> Result<Relay<'a>> {
        let mut can_share = FilterCount::default();
        let mut correct_ports = FilterCount::default();
        match &self.inner {
            ExitPathBuilderInner::AnyExit { strict } => {
                let exit = netdir.pick_relay(rng, WeightRole::Exit, |r| {
                    can_share.count(r.policies_allow_some_port())
                        && correct_ports.count(relays_can_share_circuit_opt(r, guard, config))
                });
                match (exit, strict) {
                    (Some(exit), _) => return Ok(exit),
                    (None, true) => {
                        return Err(Error::NoExit {
                            can_share,
                            correct_ports,
                        })
                    }
                    (None, false) => {}
                }

                // Non-strict case.  Arguably this doesn't belong in
                // ExitPathBuilder.
                netdir
                    .pick_relay(rng, WeightRole::Exit, |r| {
                        can_share.count(relays_can_share_circuit_opt(r, guard, config))
                    })
                    .ok_or(Error::NoExit {
                        can_share,
                        correct_ports,
                    })
            }

            ExitPathBuilderInner::WantsPorts(wantports) => Ok(netdir
                .pick_relay(rng, WeightRole::Exit, |r| {
                    can_share.count(relays_can_share_circuit_opt(r, guard, config))
                        && correct_ports.count(wantports.iter().all(|p| p.is_supported_by(r)))
                })
                .ok_or(Error::NoExit {
                    can_share,
                    correct_ports,
                })?),

            ExitPathBuilderInner::ChosenExit(exit_relay) => {
                // NOTE that this doesn't check
                // relays_can_share_circuit_opt(exit_relay,guard).  we
                // already did that, sort of, in pick_path.
                Ok(exit_relay.clone())
            }
        }
    }

    /// Try to create and return a path corresponding to the requirements of
    /// this builder.
    pub fn pick_path<R: Rng, RT: Runtime>(
        &self,
        rng: &mut R,
        netdir: DirInfo<'a>,
        guards: Option<&GuardMgr<RT>>,
        config: &PathConfig,
        _now: SystemTime,
    ) -> Result<(TorPath<'a>, Option<GuardMonitor>, Option<GuardUsable>)> {
        let netdir = match netdir {
            DirInfo::Directory(d) => d,
            _ => {
                return Err(bad_api_usage!(
                    "Tried to build a multihop path without a network directory"
                )
                .into())
            }
        };
        let subnet_config = config.subnet_config();

        let chosen_exit = if let ExitPathBuilderInner::ChosenExit(e) = &self.inner {
            Some(e)
        } else {
            None
        };
        let path_is_fully_random = chosen_exit.is_none();

        // TODO-SPEC: Because of limitations in guard selection, we have to
        // pick the guard before the exit, which is not what our spec says.
        let (guard, mon, usable) = match guards {
            Some(guardmgr) => {
                let mut b = tor_guardmgr::GuardUsageBuilder::default();
                b.kind(tor_guardmgr::GuardUsageKind::Data);
                guardmgr.update_network(netdir); // possibly unnecessary.
                if let Some(exit_relay) = chosen_exit {
                    // TODO(nickm): Our way of building a family here is
                    // somewhat questionable. We're only adding the ed25519
                    // identities of the exit relay and its family to the
                    // RelayId set.  That's fine for now, since we will only use
                    // relays at this point if they have a known Ed25519
                    // identity.  But if in the future the ed25519 identity
                    // becomes optional, this will need to change.
                    let mut family = RelayIdSet::new();
                    family.insert(*exit_relay.id());
                    // TODO(nickm): See "limitations" note on `known_family_members`.
                    family.extend(netdir.known_family_members(exit_relay).map(|r| *r.id()));
                    b.restrictions()
                        .push(tor_guardmgr::GuardRestriction::AvoidAllIds(family));
                }
                let guard_usage = b.build().expect("Failed while building guard usage!");
                let (guard, mut mon, usable) = guardmgr.select_guard(guard_usage, Some(netdir))?;
                let guard = guard.get_relay(netdir).ok_or_else(|| {
                    internal!(
                        "Somehow the guardmgr gave us an unlisted guard {:?}!",
                        guard
                    )
                })?;
                if !path_is_fully_random {
                    // We were given a specific exit relay to use, and
                    // the choice of exit relay might be forced by
                    // something outside of our control.
                    //
                    // Therefore, we must not blame the guard for any failure
                    // to complete the circuit.
                    mon.ignore_indeterminate_status();
                }
                (guard, Some(mon), Some(usable))
            }
            None => {
                let mut can_share = FilterCount::default();
                let mut correct_usage = FilterCount::default();
                let entry = netdir
                    .pick_relay(rng, WeightRole::Guard, |r| {
                        can_share.count(relays_can_share_circuit_opt(r, chosen_exit, subnet_config))
                            && correct_usage.count(r.is_flagged_guard())
                    })
                    .ok_or(Error::NoPath {
                        role: "entry relay",
                        can_share,
                        correct_usage,
                    })?;
                (entry, None, None)
            }
        };

        let exit = self.pick_exit(rng, netdir, Some(&guard), subnet_config)?;

        let mut can_share = FilterCount::default();
        let mut correct_usage = FilterCount::default();
        let middle = netdir
            .pick_relay(rng, WeightRole::Middle, |r| {
                can_share.count(
                    relays_can_share_circuit(r, &exit, subnet_config)
                        && relays_can_share_circuit(r, &guard, subnet_config),
                ) && correct_usage.count(true)
            })
            .ok_or(Error::NoPath {
                role: "middle relay",
                can_share,
                correct_usage,
            })?;

        Ok((
            TorPath::new_multihop(vec![guard, middle, exit]),
            mon,
            usable,
        ))
    }
}

/// Returns true if both relays can appear together in the same circuit.
fn relays_can_share_circuit(a: &Relay<'_>, b: &Relay<'_>, subnet_config: SubnetConfig) -> bool {
    !a.in_same_family(b) && !a.in_same_subnet(b, &subnet_config)
}

/// Helper: wraps relays_can_share_circuit but takes an option.
fn relays_can_share_circuit_opt(r1: &Relay<'_>, r2: Option<&Relay<'_>>, c: SubnetConfig) -> bool {
    match r2 {
        Some(r2) => relays_can_share_circuit(r1, r2, c),
        None => true,
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::clone_on_copy)]
    use super::*;
    use crate::path::{assert_same_path_when_owned, OwnedPath, TorPathInner};
    use crate::test::OptDummyGuardMgr;
    use std::collections::HashSet;
    use tor_basic_utils::test_rng::testing_rng;
    use tor_linkspec::{HasRelayIds, RelayIds};
    use tor_llcrypto::pk::ed25519::Ed25519Identity;
    use tor_netdir::testnet;
    use tor_rtcompat::SleepProvider;

    fn assert_exit_path_ok(relays: &[Relay<'_>]) {
        assert_eq!(relays.len(), 3);

        // TODO: Eventually assert that r1 has Guard, once we enforce that.

        let r1 = &relays[0];
        let r2 = &relays[1];
        let r3 = &relays[2];

        assert!(!r1.same_relay_ids(r2));
        assert!(!r1.same_relay_ids(r3));
        assert!(!r2.same_relay_ids(r3));

        let subnet_config = SubnetConfig::default();
        assert!(relays_can_share_circuit(r1, r2, subnet_config));
        assert!(relays_can_share_circuit(r1, r3, subnet_config));
        assert!(relays_can_share_circuit(r2, r3, subnet_config));
    }

    #[test]
    fn by_ports() {
        let mut rng = testing_rng();
        let netdir = testnet::construct_netdir().unwrap_if_sufficient().unwrap();
        let ports = vec![TargetPort::ipv4(443), TargetPort::ipv4(1119)];
        let dirinfo = (&netdir).into();
        let config = PathConfig::default();
        let guards: OptDummyGuardMgr<'_> = None;
        let now = SystemTime::now();

        for _ in 0..1000 {
            let (path, _, _) = ExitPathBuilder::from_target_ports(ports.clone())
                .pick_path(&mut rng, dirinfo, guards, &config, now)
                .unwrap();

            assert_same_path_when_owned(&path);

            if let TorPathInner::Path(p) = path.inner {
                assert_exit_path_ok(&p[..]);
                let exit = &p[2];
                assert!(exit.ipv4_policy().allows_port(1119));
            } else {
                panic!("Generated the wrong kind of path");
            }
        }

        let chosen = netdir.by_id(&Ed25519Identity::from([0x20; 32])).unwrap();

        let config = PathConfig::default();
        for _ in 0..1000 {
            let (path, _, _) = ExitPathBuilder::from_chosen_exit(chosen.clone())
                .pick_path(&mut rng, dirinfo, guards, &config, now)
                .unwrap();
            assert_same_path_when_owned(&path);
            if let TorPathInner::Path(p) = path.inner {
                assert_exit_path_ok(&p[..]);
                let exit = &p[2];
                assert!(exit.same_relay_ids(&chosen));
            } else {
                panic!("Generated the wrong kind of path");
            }
        }
    }

    #[test]
    fn any_exit() {
        let mut rng = testing_rng();
        let netdir = testnet::construct_netdir().unwrap_if_sufficient().unwrap();
        let dirinfo = (&netdir).into();
        let guards: OptDummyGuardMgr<'_> = None;
        let now = SystemTime::now();

        let config = PathConfig::default();
        for _ in 0..1000 {
            let (path, _, _) = ExitPathBuilder::for_any_exit()
                .pick_path(&mut rng, dirinfo, guards, &config, now)
                .unwrap();
            assert_same_path_when_owned(&path);
            if let TorPathInner::Path(p) = path.inner {
                assert_exit_path_ok(&p[..]);
                let exit = &p[2];
                assert!(exit.policies_allow_some_port());
            } else {
                panic!("Generated the wrong kind of path");
            }
        }
    }

    #[test]
    fn empty_path() {
        // This shouldn't actually be constructable IRL, but let's test to
        // make sure our code can handle it.
        let bogus_path = TorPath {
            inner: TorPathInner::Path(vec![]),
        };

        assert!(bogus_path.exit_relay().is_none());
        assert!(bogus_path.exit_policy().is_none());
        assert_eq!(bogus_path.len(), 0);

        let owned: Result<OwnedPath> = (&bogus_path).try_into();
        assert!(owned.is_err());
    }

    #[test]
    fn no_exits() {
        // Construct a netdir with no exits.
        let netdir = testnet::construct_custom_netdir(|_idx, bld| {
            bld.md.parse_ipv4_policy("reject 1-65535").unwrap();
        })
        .unwrap()
        .unwrap_if_sufficient()
        .unwrap();
        let mut rng = testing_rng();
        let dirinfo = (&netdir).into();
        let guards: OptDummyGuardMgr<'_> = None;
        let config = PathConfig::default();
        let now = SystemTime::now();

        // With target ports
        let outcome = ExitPathBuilder::from_target_ports(vec![TargetPort::ipv4(80)])
            .pick_path(&mut rng, dirinfo, guards, &config, now);
        assert!(outcome.is_err());
        assert!(matches!(outcome, Err(Error::NoExit { .. })));

        // For any exit
        let outcome =
            ExitPathBuilder::for_any_exit().pick_path(&mut rng, dirinfo, guards, &config, now);
        assert!(outcome.is_err());
        assert!(matches!(outcome, Err(Error::NoExit { .. })));

        // For any exit (non-strict, so this will work).
        let outcome = ExitPathBuilder::for_timeout_testing()
            .pick_path(&mut rng, dirinfo, guards, &config, now);
        assert!(outcome.is_ok());
    }

    #[test]
    fn exitpath_with_guards() {
        use tor_guardmgr::GuardStatus;

        tor_rtcompat::test_with_all_runtimes!(|rt| async move {
            let netdir = testnet::construct_netdir().unwrap_if_sufficient().unwrap();
            let mut rng = testing_rng();
            let dirinfo = (&netdir).into();
            let statemgr = tor_persist::TestingStateMgr::new();
            let guards = tor_guardmgr::GuardMgr::new(rt.clone(), statemgr, [].into()).unwrap();
            let config = PathConfig::default();
            guards.update_network(&netdir);
            let port443 = TargetPort::ipv4(443);

            // We're going to just have these all succeed and make sure
            // that they pick the same guard.  We won't test failing
            // cases here, since those are tested in guardmgr.
            let mut distinct_guards = HashSet::new();
            let mut distinct_mid = HashSet::new();
            let mut distinct_exit = HashSet::new();
            for _ in 0..20 {
                let (path, mon, usable) = ExitPathBuilder::from_target_ports(vec![port443])
                    .pick_path(&mut rng, dirinfo, Some(&guards), &config, rt.wallclock())
                    .unwrap();
                assert_eq!(path.len(), 3);
                assert_same_path_when_owned(&path);
                if let TorPathInner::Path(p) = path.inner {
                    assert_exit_path_ok(&p[..]);
                    distinct_guards.insert(RelayIds::from_relay_ids(&p[0]));
                    distinct_mid.insert(RelayIds::from_relay_ids(&p[1]));
                    distinct_exit.insert(RelayIds::from_relay_ids(&p[2]));
                } else {
                    panic!("Wrong kind of path");
                }
                let mon = mon.unwrap();
                assert!(matches!(
                    mon.inspect_pending_status(),
                    (GuardStatus::AttemptAbandoned, false)
                ));
                mon.succeeded();
                assert!(usable.unwrap().await.unwrap());
            }
            assert_eq!(distinct_guards.len(), 1);
            assert_ne!(distinct_mid.len(), 1);
            assert_ne!(distinct_exit.len(), 1);

            let guard_relay = netdir
                .by_ids(distinct_guards.iter().next().unwrap())
                .unwrap();
            let exit_relay = netdir.by_ids(distinct_exit.iter().next().unwrap()).unwrap();

            // Now we'll try a forced exit that is not the same as our
            // actual guard.
            let (path, mon, usable) = ExitPathBuilder::from_chosen_exit(exit_relay.clone())
                .pick_path(&mut rng, dirinfo, Some(&guards), &config, rt.wallclock())
                .unwrap();
            assert_eq!(path.len(), 3);
            if let TorPathInner::Path(p) = path.inner {
                assert_exit_path_ok(&p[..]);
                // We get our regular guard and our chosen exit.
                assert_eq!(p[0].ed_identity(), guard_relay.ed_identity());
                assert_eq!(p[2].ed_identity(), exit_relay.ed_identity());
            } else {
                panic!("Wrong kind of path");
            }
            let mon = mon.unwrap();
            // This time, "ignore indeterminate status" was set to true.
            assert!(matches!(
                mon.inspect_pending_status(),
                (GuardStatus::AttemptAbandoned, true)
            ));
            mon.succeeded();
            assert!(usable.unwrap().await.unwrap());

            // Finally, try with our exit forced to be our regular guard,
            // and make sure we get a different guard.
            let (path, mon, usable) = ExitPathBuilder::from_chosen_exit(guard_relay.clone())
                .pick_path(&mut rng, dirinfo, Some(&guards), &config, rt.wallclock())
                .unwrap();
            assert_eq!(path.len(), 3);
            if let TorPathInner::Path(p) = path.inner {
                // This is no longer guaranteed; see arti#183 :(
                // assert_exit_path_ok(&p[..]);
                // We get our chosen exit, and a different guard.
                assert_ne!(p[0].ed_identity(), guard_relay.ed_identity());
                assert_eq!(p[2].ed_identity(), guard_relay.ed_identity());
            } else {
                panic!("Wrong kind of path");
            }
            let mon = mon.unwrap();
            // This time, "ignore indeterminate status" was set to true.
            assert!(matches!(
                mon.inspect_pending_status(),
                (GuardStatus::AttemptAbandoned, true)
            ));
            mon.succeeded();
            assert!(usable.unwrap().await.unwrap());
        });
    }
}
