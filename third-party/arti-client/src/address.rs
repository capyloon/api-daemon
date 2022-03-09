//! Types and traits for converting objects to addresses which
//! Tor can connect to.

use crate::err::ErrorDetail;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::str::FromStr;
use thiserror::Error;

// ----------------------------------------------------------------------

/// An object that can be converted to a [`TorAddr`] with a minimum of risk.
///
/// Typically, this trait will be implemented for a hostname or service name.
///
/// Don't implement this trait for IP addresses and similar types; instead,
/// implement [`DangerouslyIntoTorAddr`] for those.  (The trouble with accepting
/// IP addresses is that, in order to get an IP address, most programs will do a
/// local hostname lookup, which will leak the target address to the DNS
/// resolver. The `DangerouslyIntoTorAddr` trait provides a contract for careful
/// programs to say, "I have gotten this IP address from somewhere safe."  This
/// trait is for name-based addressing and similar, which _usually_ gets its
/// addresses from a safer source.)
///
/// [*See also: the `TorAddr` documentation.*](TorAddr)
///
/// # Design note
///
/// We use a separate trait here, instead of using `Into<TorAddr>` or
/// `TryInto<TorAddr>`, because `IntoTorAddr` implies additional guarantees
/// relating to privacy risk.  The separate trait alerts users that something
/// tricky is going on here, and encourages them to think twice before
/// implementing `IntoTorAddr` for their own types.
pub trait IntoTorAddr {
    /// Try to make a [`TorAddr`] to represent connecting to this
    /// address.
    fn into_tor_addr(self) -> Result<TorAddr, TorAddrError>;
}

/// An object that can be converted to a [`TorAddr`], but which it
/// might be risky to get in the first place if you're hoping for
/// anonymity.
///
/// For example, you can use this trait to convert a [`SocketAddr`]
/// into a [`TorAddr`], and it's safe to do that conversion.  But
/// where did you get the [`SocketAddr`] in the first place?  If it
/// comes from a local DNS lookup, then you have leaked the address
/// you were resolving to your DNS resolver, and probably your ISP.
///
/// [*See also: the `TorAddr` documentation.*](TorAddr)
pub trait DangerouslyIntoTorAddr {
    /// Try to make a [`TorAddr`] to represent connecting to `self`.
    ///
    /// By calling this function, the caller asserts that `self` was
    /// obtained from some secure, private mechanism, and **not** from a local
    /// DNS lookup or something similar.
    fn into_tor_addr_dangerously(self) -> Result<TorAddr, TorAddrError>;
}

/// An address object that you can connect to over the Tor network.
///
/// When you're making a connection with Tor, you shouldn't do your DNS
/// lookups locally: that would leak your target address to your DNS server.
/// Instead, it's better to use a combination of a hostname and a port
/// directly.
///
/// The preferred way to create a `TorAddr` is via the [`IntoTorAddr`] trait,
/// using a hostname and a port (or a string containing a hostname and a
/// port).  It's also okay to use an IP and Port there, but only if they come
/// from some source _other than_ a local DNS lookup.
///
/// In order to discourage local hostname lookups, the functions that
/// construct a [`TorAddr`] from [`IpAddr`], [`SocketAddr`], and so
/// forth are labeled as "dangerous".
///
/// # Examples
///
/// Making a `TorAddr` from various "safe" sources:
///
/// ```rust
/// # use anyhow::Result;
/// # fn main() -> Result<()> {
/// use arti_client::IntoTorAddr;
///
/// let example_from_tuple = ("example.com", 80).into_tor_addr()?;
/// let example_from_string = "example.com:80".into_tor_addr()?;
///
/// assert_eq!(example_from_tuple, example_from_string);
/// # Ok(())
/// # }
/// ```
///
/// Making a `TorAddr` from an IP address and port:
///
/// > **Warning:** This example is only safe because we're not doing a DNS lookup; rather, the
/// > intent is to connect to a hardcoded IP address.
/// > If you're using [`DangerouslyIntoTorAddr`], pay careful attention to where your IP addresses
/// > are coming from, and whether there's a risk of information leakage.
///
/// ```rust
/// # use anyhow::Result;
/// # fn main() -> Result<()> {
/// use arti_client::DangerouslyIntoTorAddr;
/// use std::net::{IpAddr, SocketAddr};
///
/// let quad_one_dns: SocketAddr = "1.1.1.1:53".parse()?;
/// let addr_from_socketaddr = quad_one_dns.into_tor_addr_dangerously()?;
///
/// let quad_one_ip: IpAddr = "1.1.1.1".parse()?;
/// let addr_from_tuple = (quad_one_ip, 53).into_tor_addr_dangerously()?;
///
/// assert_eq!(addr_from_socketaddr, addr_from_tuple);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TorAddr {
    /// The target host.
    host: Host,
    /// The target port number.
    port: u16,
}

impl TorAddr {
    /// Construct a TorAddr from its constituent parts, rejecting it if the
    /// port is zero.
    fn new(host: Host, port: u16) -> Result<Self, TorAddrError> {
        if port == 0 {
            Err(TorAddrError::BadPort)
        } else {
            Ok(TorAddr { host, port })
        }
    }

    /// Construct a `TorAddr` from any object that implements
    /// [`IntoTorAddr`].
    pub fn from<A: IntoTorAddr>(addr: A) -> Result<Self, TorAddrError> {
        addr.into_tor_addr()
    }
    /// Construct a `TorAddr` from any object that implements
    /// [`DangerouslyIntoTorAddr`].
    ///
    /// See [`DangerouslyIntoTorAddr`] for an explanation of why the
    /// style of programming supported by this function is dangerous
    /// to use.
    pub fn dangerously_from<A: DangerouslyIntoTorAddr>(addr: A) -> Result<Self, TorAddrError> {
        addr.into_tor_addr_dangerously()
    }

    /// Return true if this is an IP address (rather than a hostname).
    pub fn is_ip_address(&self) -> bool {
        matches!(&self.host, Host::Ip(_))
    }

    /// Extract a `String`-based hostname and a `u16` port from this
    /// address.
    pub(crate) fn into_string_and_port(self) -> (String, u16) {
        let host = self.host.to_string();
        let port = self.port;
        (host, port)
    }

    /// Return true if the `host` in this address is local.
    fn is_local(&self) -> bool {
        self.host.is_local()
    }

    /// Give an error if this address doesn't conform to the rules set in
    /// `cfg`.
    pub(crate) fn enforce_config(
        &self,
        cfg: &crate::config::ClientAddrConfig,
    ) -> Result<(), ErrorDetail> {
        if !cfg.allow_local_addrs && self.is_local() {
            return Err(ErrorDetail::LocalAddress);
        }

        if let Host::Hostname(addr) = &self.host {
            if !is_valid_hostname(addr) {
                return Err(ErrorDetail::InvalidHostname);
            }
            if addr.to_lowercase().ends_with(".onion") {
                return Err(ErrorDetail::OnionAddressNotSupported);
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for TorAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.host {
            Host::Ip(IpAddr::V6(addr)) => write!(f, "[{}]:{}", addr, self.port),
            _ => write!(f, "{}:{}", self.host, self.port),
        }
    }
}

/// An error created while making or using a [`TorAddr`].
#[derive(Debug, Error, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum TorAddrError {
    /// Tried to parse a string that can never be interpreted as a valid host.
    #[error("String can never be a valid hostname")]
    InvalidHostname,
    /// Tried to parse a string as an `address:port`, but it had no port.
    #[error("No port found in string")]
    NoPort,
    /// Tried to parse a port that wasn't a valid nonzero `u16`.
    #[error("Could not parse port")]
    BadPort,
}

/// A host that Tor can connect to: either a hostname or an IP address.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Host {
    /// A hostname.  This variant should never be used if the `Ip`
    /// variant could be used instead.
    Hostname(String),
    /// An IP address.
    Ip(IpAddr),
}

impl FromStr for Host {
    type Err = TorAddrError;
    fn from_str(s: &str) -> Result<Host, TorAddrError> {
        if let Ok(ip_addr) = s.parse() {
            Ok(Host::Ip(ip_addr))
        } else {
            // TODO(nickm): we might someday want to reject some kinds of bad
            // hostnames here, rather than when we're about to connect to them.
            // But that would be an API break, and maybe not what people want.
            // Maybe instead we should have a method to check whether a hostname
            // is "bad"? Not sure; we'll need to decide the right behavior here.
            Ok(Host::Hostname(s.to_owned()))
        }
    }
}

impl Host {
    /// Return true if this address is one that is "internal": that is,
    /// relative to the particular host that is resolving it.
    fn is_local(&self) -> bool {
        match self {
            Host::Hostname(name) => name.eq_ignore_ascii_case("localhost"),
            // TODO: use is_global once it's stable.
            Host::Ip(IpAddr::V4(ip)) => ip.is_loopback() || ip.is_private(),
            Host::Ip(IpAddr::V6(ip)) => ip.is_loopback(),
        }
    }
}

impl std::fmt::Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Host::Hostname(s) => write!(f, "{}", s),
            Host::Ip(ip) => write!(f, "{}", ip),
        }
    }
}

impl IntoTorAddr for TorAddr {
    fn into_tor_addr(self) -> Result<TorAddr, TorAddrError> {
        Ok(self)
    }
}

impl<A: IntoTorAddr + Clone> IntoTorAddr for &A {
    fn into_tor_addr(self) -> Result<TorAddr, TorAddrError> {
        self.clone().into_tor_addr()
    }
}

impl IntoTorAddr for &str {
    fn into_tor_addr(self) -> Result<TorAddr, TorAddrError> {
        if let Ok(sa) = SocketAddr::from_str(self) {
            TorAddr::new(Host::Ip(sa.ip()), sa.port())
        } else {
            let (host, port) = self.rsplit_once(':').ok_or(TorAddrError::NoPort)?;
            let host = host.parse()?;
            let port = port.parse().map_err(|_| TorAddrError::BadPort)?;
            TorAddr::new(host, port)
        }
    }
}

impl IntoTorAddr for String {
    fn into_tor_addr(self) -> Result<TorAddr, TorAddrError> {
        self[..].into_tor_addr()
    }
}

impl FromStr for TorAddr {
    type Err = TorAddrError;
    fn from_str(s: &str) -> Result<Self, TorAddrError> {
        s.into_tor_addr()
    }
}

impl IntoTorAddr for (&str, u16) {
    fn into_tor_addr(self) -> Result<TorAddr, TorAddrError> {
        let (host, port) = self;
        let host = host.parse()?;
        TorAddr::new(host, port)
    }
}

impl IntoTorAddr for (String, u16) {
    fn into_tor_addr(self) -> Result<TorAddr, TorAddrError> {
        let (host, port) = self;
        (&host[..], port).into_tor_addr()
    }
}

impl<T: DangerouslyIntoTorAddr + Clone> DangerouslyIntoTorAddr for &T {
    fn into_tor_addr_dangerously(self) -> Result<TorAddr, TorAddrError> {
        self.clone().into_tor_addr_dangerously()
    }
}

impl DangerouslyIntoTorAddr for (IpAddr, u16) {
    fn into_tor_addr_dangerously(self) -> Result<TorAddr, TorAddrError> {
        let (addr, port) = self;
        TorAddr::new(Host::Ip(addr), port)
    }
}

impl DangerouslyIntoTorAddr for (Ipv4Addr, u16) {
    fn into_tor_addr_dangerously(self) -> Result<TorAddr, TorAddrError> {
        let (addr, port) = self;
        TorAddr::new(Host::Ip(addr.into()), port)
    }
}

impl DangerouslyIntoTorAddr for (Ipv6Addr, u16) {
    fn into_tor_addr_dangerously(self) -> Result<TorAddr, TorAddrError> {
        let (addr, port) = self;
        TorAddr::new(Host::Ip(addr.into()), port)
    }
}

impl DangerouslyIntoTorAddr for SocketAddr {
    fn into_tor_addr_dangerously(self) -> Result<TorAddr, TorAddrError> {
        let (addr, port) = (self.ip(), self.port());
        (addr, port).into_tor_addr_dangerously()
    }
}

impl DangerouslyIntoTorAddr for SocketAddrV4 {
    fn into_tor_addr_dangerously(self) -> Result<TorAddr, TorAddrError> {
        let (addr, port) = (self.ip(), self.port());
        (*addr, port).into_tor_addr_dangerously()
    }
}

impl DangerouslyIntoTorAddr for SocketAddrV6 {
    fn into_tor_addr_dangerously(self) -> Result<TorAddr, TorAddrError> {
        let (addr, port) = (self.ip(), self.port());
        (*addr, port).into_tor_addr_dangerously()
    }
}

/// Check whether `hostname` is a valid hostname or not.
///
/// (Note that IPv6 addresses don't follow these rules.)
///
/// TODO: Check whether the rules given here are in fact the same rules
/// as Tor follows, and whether they conform to anything.
fn is_valid_hostname(hostname: &str) -> bool {
    /// Check if we have the valid characters for a hostname
    fn is_valid_char(byte: u8) -> bool {
        ((b'a'..=b'z').contains(&byte))
            || ((b'A'..=b'Z').contains(&byte))
            || ((b'0'..=b'9').contains(&byte))
            || byte == b'-'
            || byte == b'.'
    }

    !(hostname.bytes().any(|byte| !is_valid_char(byte))
        || hostname.ends_with('-')
        || hostname.starts_with('-')
        || hostname.ends_with('.')
        || hostname.starts_with('.')
        || hostname.is_empty())
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn validate_hostname() {
        // Valid hostname tests
        assert!(is_valid_hostname("torproject.org"));
        assert!(is_valid_hostname("Tor-Project.org"));

        // Invalid hostname tests
        assert!(!is_valid_hostname("-torproject.org"));
        assert!(!is_valid_hostname("_torproject.org"));
        assert!(!is_valid_hostname("tor_project1.org"));
        assert!(!is_valid_hostname("iwanna$money.org"));
        assert!(!is_valid_hostname(""));
    }

    #[test]
    fn validate_addr() {
        use crate::err::ErrorDetail;
        fn val<A: IntoTorAddr>(addr: A) -> Result<TorAddr, ErrorDetail> {
            let toraddr = addr.into_tor_addr()?;
            toraddr.enforce_config(&Default::default())?;
            Ok(toraddr)
        }

        assert!(val("[2001:db8::42]:20").is_ok());
        assert!(val(("2001:db8::42", 20)).is_ok());
        assert!(val(("198.151.100.42", 443)).is_ok());
        assert!(val("198.151.100.42:443").is_ok());
        assert!(val("www.torproject.org:443").is_ok());
        assert!(val(("www.torproject.org", 443)).is_ok());

        assert!(matches!(
            val("-foobar.net:443"),
            Err(ErrorDetail::InvalidHostname)
        ));
        assert!(matches!(
            val("www.torproject.org"),
            Err(ErrorDetail::Address(TorAddrError::NoPort))
        ));

        assert!(matches!(
            val("192.168.0.1:80"),
            Err(ErrorDetail::LocalAddress)
        ));
        assert!(matches!(
            val("eweiibe6tdjsdprb4px6rqrzzcsi22m4koia44kc5pcjr7nec2rlxyad.onion:443"),
            Err(ErrorDetail::OnionAddressNotSupported)
        ));
    }

    #[test]
    fn local_addrs() {
        fn is_local_hostname(s: &str) -> bool {
            let h: Host = s.parse().unwrap();
            h.is_local()
        }

        assert!(is_local_hostname("localhost"));
        assert!(is_local_hostname("loCALHOST"));
        assert!(is_local_hostname("127.0.0.1"));
        assert!(is_local_hostname("::1"));
        assert!(is_local_hostname("192.168.0.1"));

        assert!(!is_local_hostname("www.example.com"));
    }

    #[test]
    fn is_ip_address() {
        fn ip(s: &str) -> bool {
            TorAddr::from(s).unwrap().is_ip_address()
        }

        assert!(ip("192.168.0.1:80"));
        assert!(ip("[::1]:80"));
        assert!(ip("[2001:db8::42]:65535"));
        assert!(!ip("example.com:80"));
        assert!(!ip("example.onion:80"));
    }

    #[test]
    fn string_and_port() {
        fn sap(s: &str) -> (String, u16) {
            TorAddr::from(s).unwrap().into_string_and_port()
        }

        assert_eq!(
            sap("[2001:db8::42]:9001"),
            ("2001:db8::42".to_owned(), 9001)
        );
        assert_eq!(sap("example.com:80"), ("example.com".to_owned(), 80));
    }

    #[test]
    fn bad_ports() {
        assert_eq!(
            TorAddr::from("www.example.com:squirrel"),
            Err(TorAddrError::BadPort)
        );
        assert_eq!(
            TorAddr::from("www.example.com:0"),
            Err(TorAddrError::BadPort)
        );
    }

    #[test]
    fn convert_safe() {
        fn check<A: IntoTorAddr>(a: A, s: &str) {
            let a1 = TorAddr::from(a).unwrap();
            let a2 = s.parse().unwrap();
            assert_eq!(a1, a2);
            assert_eq!(&a1.to_string(), s);
        }

        check(("www.example.com", 8000), "www.example.com:8000");
        check(
            TorAddr::from(("www.example.com", 8000)).unwrap(),
            "www.example.com:8000",
        );
        check(
            &TorAddr::from(("www.example.com", 8000)).unwrap(),
            "www.example.com:8000",
        );
        check("[2001:db8::0042]:9001".to_owned(), "[2001:db8::42]:9001");
        check(("2001:db8::0042".to_owned(), 9001), "[2001:db8::42]:9001");
    }

    #[test]
    fn convert_dangerous() {
        fn check<A: DangerouslyIntoTorAddr>(a: A, s: &str) {
            let a1 = TorAddr::dangerously_from(a).unwrap();
            let a2 = TorAddr::from(s).unwrap();
            assert_eq!(a1, a2);
            assert_eq!(&a1.to_string(), s);
        }

        let ip: IpAddr = "203.0.133.6".parse().unwrap();
        let ip4: Ipv4Addr = "203.0.133.7".parse().unwrap();
        let ip6: Ipv6Addr = "2001:db8::42".parse().unwrap();
        let sa: SocketAddr = "203.0.133.8:80".parse().unwrap();
        let sa4: SocketAddrV4 = "203.0.133.8:81".parse().unwrap();
        let sa6: SocketAddrV6 = "[2001:db8::43]:82".parse().unwrap();

        check(&(ip, 443), "203.0.133.6:443");
        check((ip, 443), "203.0.133.6:443");
        check((ip4, 444), "203.0.133.7:444");
        check((ip6, 445), "[2001:db8::42]:445");
        check(sa, "203.0.133.8:80");
        check(sa4, "203.0.133.8:81");
        check(sa6, "[2001:db8::43]:82");
    }
}
