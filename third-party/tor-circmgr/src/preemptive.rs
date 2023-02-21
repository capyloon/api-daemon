//! Tools for determining what circuits to preemptively build.

use crate::{PreemptiveCircuitConfig, TargetCircUsage, TargetPort};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::warn;

/// Predicts what circuits might be used in future based on past activity, and suggests
/// circuits to preemptively build as a result.
pub(crate) struct PreemptiveCircuitPredictor {
    /// A map of every exit port we've observed being used (or `None` if we observed an exit being
    /// used to resolve DNS names instead of building a stream), to the last time we encountered
    /// such usage.
    // TODO(nickm): Let's have a mechanism for cleaning this out from time to time.
    usages: HashMap<Option<TargetPort>, Instant>,

    /// Configuration for this predictor.
    config: tor_config::MutCfg<PreemptiveCircuitConfig>,
}

impl PreemptiveCircuitPredictor {
    /// Create a new predictor, starting out with a set of ports we think are likely to be used.
    pub(crate) fn new(config: PreemptiveCircuitConfig) -> Self {
        let mut usages = HashMap::new();
        for port in &config.initial_predicted_ports {
            // TODO(nickm) should this be IPv6? Should we have a way to configure IPv6 initial ports?
            usages.insert(Some(TargetPort::ipv4(*port)), Instant::now());
        }

        // We want to build circuits for resolving DNS, too.
        usages.insert(None, Instant::now());

        Self {
            usages,
            config: config.into(),
        }
    }

    /// Return the configuration for this PreemptiveCircuitPredictor.
    pub(crate) fn config(&self) -> Arc<PreemptiveCircuitConfig> {
        self.config.get()
    }

    /// Replace the current configuration for this PreemptiveCircuitPredictor
    /// with `new_config`.
    pub(crate) fn set_config(&self, mut new_config: PreemptiveCircuitConfig) {
        self.config.map_and_replace(|cfg| {
            // Force this to stay the same, since it can't meaningfully be changed.
            new_config.initial_predicted_ports = cfg.initial_predicted_ports.clone();
            new_config
        });
    }

    /// Make some predictions for what circuits should be built.
    pub(crate) fn predict(&self) -> Vec<TargetCircUsage> {
        let config = self.config();
        let now = Instant::now();
        let circs = config.min_exit_circs_for_port;
        self.usages
            .iter()
            .filter(|(_, &time)| {
                time.checked_add(config.prediction_lifetime)
                    .map(|t| t > now)
                    .unwrap_or_else(|| {
                        // FIXME(eta): this is going to be a bit noisy if it triggers, but that's better
                        //             than panicking or silently doing the wrong thing?
                        warn!("failed to represent preemptive circuit prediction lifetime as an Instant");
                        false
                    })
            })
            .map(|(&port, _)| TargetCircUsage::Preemptive { port, circs })
            .collect()
    }

    /// Note the use of a new port at the provided `time`.
    ///
    /// # Limitations
    ///
    /// This function assumes that the `time` values it receives are
    /// monotonically increasing.
    pub(crate) fn note_usage(&mut self, port: Option<TargetPort>, time: Instant) {
        self.usages.insert(port, time);
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use crate::{PreemptiveCircuitConfig, PreemptiveCircuitPredictor, TargetCircUsage, TargetPort};
    use std::time::{Duration, Instant};

    use crate::isolation::test::{assert_isoleq, IsolationTokenEq};

    #[test]
    fn predicts_starting_ports() {
        let mut cfg = PreemptiveCircuitConfig::builder();
        cfg.set_initial_predicted_ports(vec![]);
        cfg.prediction_lifetime(Duration::from_secs(2));
        let predictor = PreemptiveCircuitPredictor::new(cfg.build().unwrap());

        assert_isoleq!(
            predictor.predict(),
            vec![TargetCircUsage::Preemptive {
                port: None,
                circs: 2
            }]
        );

        let mut cfg = PreemptiveCircuitConfig::builder();
        cfg.set_initial_predicted_ports(vec![80]);
        cfg.prediction_lifetime(Duration::from_secs(2));
        let predictor = PreemptiveCircuitPredictor::new(cfg.build().unwrap());

        let results = predictor.predict();
        assert_eq!(results.len(), 2);
        assert!(results
            .iter()
            .any(|r| r.isol_eq(&TargetCircUsage::Preemptive {
                port: None,
                circs: 2
            })));
        assert!(results
            .iter()
            .any(|r| r.isol_eq(&TargetCircUsage::Preemptive {
                port: Some(TargetPort::ipv4(80)),
                circs: 2
            })));
    }

    #[test]
    fn predicts_used_ports() {
        let mut cfg = PreemptiveCircuitConfig::builder();
        cfg.set_initial_predicted_ports(vec![]);
        cfg.prediction_lifetime(Duration::from_secs(2));
        let mut predictor = PreemptiveCircuitPredictor::new(cfg.build().unwrap());

        assert_isoleq!(
            predictor.predict(),
            vec![TargetCircUsage::Preemptive {
                port: None,
                circs: 2
            }]
        );

        predictor.note_usage(Some(TargetPort::ipv4(1234)), Instant::now());

        let results = predictor.predict();
        assert_eq!(results.len(), 2);
        assert!(results
            .iter()
            .any(|r| r.isol_eq(&TargetCircUsage::Preemptive {
                port: None,
                circs: 2
            })));
        assert!(results
            .iter()
            .any(|r| r.isol_eq(&TargetCircUsage::Preemptive {
                port: Some(TargetPort::ipv4(1234)),
                circs: 2
            })));
    }

    #[test]
    fn does_not_predict_old_ports() {
        let mut cfg = PreemptiveCircuitConfig::builder();
        cfg.set_initial_predicted_ports(vec![]);
        cfg.prediction_lifetime(Duration::from_secs(2));
        let mut predictor = PreemptiveCircuitPredictor::new(cfg.build().unwrap());
        let now = Instant::now();
        let three_seconds_ago = now - Duration::from_secs(2 + 1);

        predictor.note_usage(Some(TargetPort::ipv4(2345)), three_seconds_ago);

        assert_isoleq!(
            predictor.predict(),
            vec![TargetCircUsage::Preemptive {
                port: None,
                circs: 2
            }]
        );
    }
}
