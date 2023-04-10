//! Networking related utilities

use std::net::{IpAddr, Ipv6Addr, SocketAddr};

use anyhow::{ensure, Result};

const IFF_UP: u32 = 0x1;
const IFF_LOOPBACK: u32 = 0x8;

/// List of machine's IP addresses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalAddresses {
    /// Loopback addresses.
    pub loopback: Vec<IpAddr>,
    /// Regular addresses.
    pub regular: Vec<IpAddr>,
}

impl Default for LocalAddresses {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalAddresses {
    /// Returns the machine's IP addresses.
    /// If there are no regular addresses it will return any IPv4 linklocal or IPv6 unique local
    /// addresses because we know of environments where these are used with NAT to provide connectivity.
    pub fn new() -> Self {
        let ifaces = default_net::interface::get_interfaces();

        let mut loopback = Vec::new();
        let mut regular4 = Vec::new();
        let mut regular6 = Vec::new();
        let mut linklocal4 = Vec::new();
        let mut ula6 = Vec::new();

        for iface in ifaces {
            if !is_up(&iface) {
                // Skip down interfaces
                continue;
            }
            let ifc_is_loopback = is_loopback(&iface);
            let addrs = iface
                .ipv4
                .iter()
                .map(|a| IpAddr::V4(a.addr))
                .chain(iface.ipv6.iter().map(|a| IpAddr::V6(a.addr)));

            for ip in addrs {
                let ip = to_canonical(ip);

                if ip.is_loopback() || ifc_is_loopback {
                    loopback.push(ip);
                } else if is_link_local(ip) {
                    if ip.is_ipv4() {
                        linklocal4.push(ip);
                    }

                    // We know of no cases where the IPv6 fe80:: addresses
                    // are used to provide WAN connectivity. It is also very
                    // common for users to have no IPv6 WAN connectivity,
                    // but their OS supports IPv6 so they have an fe80::
                    // address. We don't want to report all of those
                    // IPv6 LL to Control.
                } else if ip.is_ipv6() && is_private(&ip) {
                    // Google Cloud Run uses NAT with IPv6 Unique
                    // Local Addresses to provide IPv6 connectivity.
                    ula6.push(ip);
                } else if ip.is_ipv4() {
                    regular4.push(ip);
                } else {
                    regular6.push(ip);
                }
            }
        }

        if regular4.is_empty() && regular6.is_empty() {
            // if we have no usable IP addresses then be willing to accept
            // addresses we otherwise wouldn't, like:
            //   + 169.254.x.x (AWS Lambda uses NAT with these)
            //   + IPv6 ULA (Google Cloud Run uses these with address translation)
            regular4 = linklocal4;
            regular6 = ula6;
        }
        let mut regular = regular4;
        regular.extend(regular6);

        regular.sort();
        loopback.sort();

        LocalAddresses { loopback, regular }
    }
}

const fn is_up(interface: &default_net::Interface) -> bool {
    interface.flags & IFF_UP != 0
}

const fn is_loopback(interface: &default_net::Interface) -> bool {
    interface.flags & IFF_LOOPBACK != 0
}

/// Reports whether ip is a private address, according to RFC 1918
/// (IPv4 addresses) and RFC 4193 (IPv6 addresses). That is, it reports whether
/// ip is in 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, or fc00::/7.
fn is_private(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            // RFC 1918 allocates 10.0.0.0/8, 172.16.0.0/12, and 192.168.0.0/16 as
            // private IPv4 address subnets.
            let octets = ip.octets();
            octets[0] == 10
                || (octets[0] == 172 && octets[1] & 0xf0 == 16)
                || (octets[0] == 192 && octets[1] == 168)
        }
        IpAddr::V6(ip) => is_private_v6(ip),
    }
}

fn is_private_v6(ip: &Ipv6Addr) -> bool {
    // RFC 4193 allocates fc00::/7 as the unique local unicast IPv6 address subnet.
    ip.octets()[0] & 0xfe == 0xfc
}

fn is_link_local(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => ip.is_link_local(),
        IpAddr::V6(ip) => is_unicast_link_local(ip),
    }
}

// TODO: replace with IpAddr::to_canoncial once stabilized.
fn to_canonical(ip: IpAddr) -> IpAddr {
    match ip {
        ip @ IpAddr::V4(_) => ip,
        IpAddr::V6(ip) => {
            if let Some(ip) = ip.to_ipv4_mapped() {
                IpAddr::V4(ip)
            } else {
                IpAddr::V6(ip)
            }
        }
    }
}
// Copied from std lib, not stable yet
const fn is_unicast_link_local(addr: Ipv6Addr) -> bool {
    (addr.segments()[0] & 0xffc0) == 0xfe80
}

/// Given a listen/bind address, finds all the local addresses for that address family.
pub(crate) fn find_local_addresses(listen_addr: SocketAddr) -> Result<Vec<SocketAddr>> {
    let listen_ip = listen_addr.ip();
    let listen_port = listen_addr.port();
    let addrs: Vec<SocketAddr> = match listen_ip.is_unspecified() {
        true => {
            // Find all the local addresses for this address family.
            let addrs = LocalAddresses::new();
            addrs
                .regular
                .iter()
                .filter(|addr| match addr {
                    IpAddr::V4(_) if listen_ip.is_ipv4() => true,
                    IpAddr::V6(_) if listen_ip.is_ipv6() => true,
                    _ => false,
                })
                .copied()
                .map(|addr| SocketAddr::from((addr, listen_port)))
                .collect()
        }
        false => vec![listen_addr],
    };
    ensure!(!addrs.is_empty(), "No local addresses found");
    Ok(addrs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_addresses() {
        let addrs = LocalAddresses::new();
        dbg!(&addrs);
        assert!(!addrs.loopback.is_empty());
        assert!(!addrs.regular.is_empty());
    }
}
