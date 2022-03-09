//! Implement port-based policies
//!
//! These are also known as "short policies" or "policy summaries".

use std::fmt::Display;
use std::str::FromStr;
use std::sync::Arc;

use super::{PolicyError, PortRange};
use crate::util::intern::InternCache;

/// A policy to match zero or more TCP/UDP ports.
///
/// These are used in Tor to summarize all policies in
/// microdescriptors, and Ipv6 policies in router descriptors.
///
/// NOTE: If a port is listed as accepted, it doesn't mean that the
/// relay allows _every_ address on that port.  Instead, a port is
/// listed if a relay will exit to _most public addresses_ on that
/// port. Therefore, unlike [super::addrpolicy::AddrPolicy] objects,
/// these policies cannot tell you if a port is _definitely_ allowed
/// or rejected: only if it is _probably_ allowed or rejected.
///
/// # Examples
/// ```
/// use tor_netdoc::types::policy::PortPolicy;
/// let policy: PortPolicy = "accept 1-1023,8000-8999,60000-65535".parse().unwrap();
///
/// assert!(policy.allows_port(22));
/// assert!(policy.allows_port(8000));
/// assert!(! policy.allows_port(1024));
/// assert!(! policy.allows_port(9000));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PortPolicy {
    /// A list of port ranges that this policy allows.
    ///
    /// These ranges sorted, disjoint, and compact.
    allowed: Vec<PortRange>,
}

impl Display for PortPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.allowed.is_empty() {
            write!(f, "reject 1-65535")?;
        } else {
            write!(f, "accept ")?;
            let mut comma = "";
            for range in &self.allowed {
                write!(f, "{}{}", comma, range)?;
                comma = ",";
            }
        }
        Ok(())
    }
}

impl PortPolicy {
    /// Return a new PortPolicy that rejects all ports.
    pub fn new_reject_all() -> Self {
        PortPolicy {
            allowed: Vec::new(),
        }
    }
    /// Helper: replace this policy with its inverse.
    fn invert(&mut self) {
        let mut prev_hi = 0;
        let mut new_allowed = Vec::new();
        for entry in &self.allowed {
            // ports prev_hi+1 through entry.lo-1 were rejected.  We should
            // make them allowed.
            if entry.lo > prev_hi + 1 {
                new_allowed.push(PortRange::new_unchecked(prev_hi + 1, entry.lo - 1));
            }
            prev_hi = entry.hi;
        }
        if prev_hi < 65535 {
            new_allowed.push(PortRange::new_unchecked(prev_hi + 1, 65535));
        }
        self.allowed = new_allowed;
    }
    /// Helper: add a new range to the end of this portpolicy.
    ///
    /// gives an error if this range cannot appear next in sequence.
    fn push_policy(&mut self, item: PortRange) -> Result<(), PolicyError> {
        if let Some(prev) = self.allowed.last() {
            // TODO SPEC: We don't enforce this in Tor, but we probably
            // should.  See torspec#60.
            if prev.hi >= item.lo {
                return Err(PolicyError::InvalidPolicy);
            } else if prev.hi == item.lo - 1 {
                // We compress a-b,(b+1)-c into a-c.
                let r = PortRange::new_unchecked(prev.lo, item.hi);
                self.allowed.pop();
                self.allowed.push(r);
                return Ok(());
            }
        }

        self.allowed.push(item);
        Ok(())
    }
    /// Return true iff `port` is allowed by this policy.
    pub fn allows_port(&self, port: u16) -> bool {
        self.allowed
            .binary_search_by(|range| range.compare_to_port(port))
            .is_ok()
    }
    /// Replace this PortPolicy with an interned copy, to save memory.
    pub fn intern(self) -> Arc<Self> {
        POLICY_CACHE.intern(self)
    }
    /// Return true if this policy allows any ports at all.
    ///
    /// # Example
    /// ```
    /// use tor_netdoc::types::policy::PortPolicy;
    ///
    /// let policy: PortPolicy = "accept 22".parse().unwrap();
    /// assert!(policy.allows_some_port());
    /// let policy2: PortPolicy = "reject 1-65535".parse().unwrap();
    /// assert!(! policy2.allows_some_port());
    /// ```
    pub fn allows_some_port(&self) -> bool {
        !self.allowed.is_empty()
    }
}

impl FromStr for PortPolicy {
    type Err = PolicyError;
    fn from_str(mut s: &str) -> Result<Self, PolicyError> {
        let invert = if s.starts_with("accept ") {
            false
        } else if s.starts_with("reject ") {
            true
        } else {
            return Err(PolicyError::InvalidPolicy);
        };
        let mut result = PortPolicy {
            allowed: Vec::new(),
        };
        s = &s[7..];
        for item in s.split(',') {
            let r: PortRange = item.parse()?;
            result.push_policy(r)?;
        }
        if invert {
            result.invert();
        }
        Ok(result)
    }
}

/// Cache of PortPolicy objects, for saving memory.
//
/// This only holds weak references to the policy objects, so we don't
/// need to worry about running out of space because of stale entries.
static POLICY_CACHE: InternCache<PortPolicy> = InternCache::new();

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn test_roundtrip() {
        fn check(inp: &str, outp: &str, allow: &[u16], deny: &[u16]) {
            let policy = inp.parse::<PortPolicy>().unwrap();
            assert_eq!(format!("{}", policy), outp);
            for p in allow {
                assert!(policy.allows_port(*p));
            }
            for p in deny {
                assert!(!policy.allows_port(*p));
            }
        }

        check(
            "accept 1-10,30-50,600",
            "accept 1-10,30-50,600",
            &[1, 10, 35, 600],
            &[0, 11, 55, 599, 601],
        );
        check("accept 1-10,11-20", "accept 1-20", &[], &[]);
        check(
            "reject 1-30",
            "accept 31-65535",
            &[31, 10001, 65535],
            &[0, 1, 30],
        );
        check(
            "reject 300-500",
            "accept 1-299,501-65535",
            &[31, 10001, 65535],
            &[300, 301, 500],
        );
        check("reject 10,11,12,13,15", "accept 1-9,14,16-65535", &[], &[]);
        check(
            "reject 1-65535",
            "reject 1-65535",
            &[],
            &[1, 300, 301, 500, 10001, 65535],
        );
    }

    #[test]
    fn test_bad() {
        for s in &[
            "ignore 1-10",
            "allow 1-100",
            "accept",
            "reject",
            "accept x-y",
            "accept 1-20,19-30",
            "accept 1-20,20-30",
            "reject 1,1,1,1",
            "reject 1,2,foo,4",
            "reject 5,4,3,2",
        ] {
            assert!(s.parse::<PortPolicy>().is_err());
        }
    }
}
