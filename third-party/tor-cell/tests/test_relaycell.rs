// Tests for encoding/decoding relay messages into relay cell bodies.
#![allow(clippy::uninlined_format_args)]

use tor_bytes::Error;
use tor_cell::relaycell::{msg, msg::RelayMsg, RelayCell, RelayCmd, StreamId};

#[cfg(feature = "experimental-udp")]
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};
#[cfg(feature = "experimental-udp")]
use tor_cell::relaycell::udp::Address;

const CELL_BODY_LEN: usize = 509;

struct BadRng;
impl rand::RngCore for BadRng {
    fn next_u32(&mut self) -> u32 {
        0xf0f0f0f0
    }
    fn next_u64(&mut self) -> u64 {
        0xf0f0f0f0f0f0f0f0
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.fill(0xf0);
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

// I won't tell if you don't.
impl rand::CryptoRng for BadRng {}

fn decode(body: &str) -> [u8; CELL_BODY_LEN] {
    let mut body = body.to_string();
    body.retain(|c| !c.is_whitespace());
    let mut body = hex::decode(body).unwrap();
    body.resize(CELL_BODY_LEN, 0xf0); // see BadRng

    let mut result = [0; CELL_BODY_LEN];
    result.copy_from_slice(&body[..]);
    result
}

fn cell(body: &str, id: StreamId, msg: RelayMsg) {
    let body = decode(body);
    let mut bad_rng = BadRng;

    let expected = RelayCell::new(id, msg);

    let decoded = RelayCell::decode(body).unwrap();

    assert_eq!(format!("{:?}", expected), format!("{:?}", decoded));

    let encoded1 = decoded.encode(&mut bad_rng).unwrap();
    let encoded2 = expected.encode(&mut bad_rng).unwrap();

    assert_eq!(&encoded1[..], &encoded2[..]);
}

#[test]
fn bad_rng() {
    use rand::RngCore;
    let mut rng = BadRng;

    assert_eq!(rng.next_u32(), 0xf0f0f0f0);
    assert_eq!(rng.next_u64(), 0xf0f0f0f0f0f0f0f0);
    let mut buf = [0u8; 19];
    assert!(rng.try_fill_bytes(&mut buf).is_ok());
    assert_eq!(
        &buf,
        &[
            0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0,
            0xf0, 0xf0, 0xf0, 0xf0, 0xf0,
        ]
    );
}

#[test]
fn test_cells() {
    cell(
        "02 0000 9999 12345678 000c 6e6565642d746f2d6b6e6f77 00000000",
        0x9999.into(),
        msg::Data::new(&b"need-to-know"[..]).unwrap().into(),
    );

    // length too big: 0x1f3 is one byte too many.
    let m = decode("02 0000 9999 12345678 01f3 6e6565642d746f2d6b6e6f77 00000000");
    assert_eq!(
        RelayCell::decode(m).err(),
        Some(Error::BadMessage("Insufficient data in relay cell"))
    );

    // check accessors.
    let m = decode("02 0000 9999 12345678 01f2 6e6565642d746f2d6b6e6f77 00000000");
    let c = RelayCell::decode(m).unwrap();
    assert_eq!(c.cmd(), RelayCmd::from(2));
    assert_eq!(c.msg().cmd(), RelayCmd::from(2));
    let (s, _) = c.into_streamid_and_msg();
    assert_eq!(s, StreamId::from(0x9999));
}

#[test]
fn test_streamid() {
    let zero: StreamId = 0.into();
    let two: StreamId = 2.into();

    assert!(zero.is_zero());
    assert!(!two.is_zero());

    assert_eq!(format!("{}", zero), "0");
    assert_eq!(format!("{}", two), "2");

    assert_eq!(u16::from(zero), 0_u16);
    assert_eq!(u16::from(two), 2_u16);

    assert!(RelayCmd::DATA.accepts_streamid_val(two));
    assert!(!RelayCmd::DATA.accepts_streamid_val(zero));

    assert!(RelayCmd::EXTEND2.accepts_streamid_val(zero));
    assert!(!RelayCmd::EXTEND2.accepts_streamid_val(two));
}

#[cfg(feature = "experimental-udp")]
#[test]
fn test_address() {
    // IPv4
    let ipv4 = Ipv4Addr::from_str("1.2.3.4").expect("Unable to parse IPv4");
    let addr = Address::from_str("1.2.3.4").expect("Unable to parse Address");
    assert!(matches!(addr, Address::Ipv4(_)));
    assert_eq!(addr, Address::Ipv4(ipv4));

    // Wrong IPv4 should result in a hostname.
    let addr = Address::from_str("1.2.3.372").expect("Unable to parse Address");
    assert!(addr.is_hostname());

    // Common bad IPv4 patterns
    let addr = Address::from_str("0x23.42.42.42").expect("Unable to parse Address");
    assert!(addr.is_hostname());
    let addr = Address::from_str("0x7f000001").expect("Unable to parse Address");
    assert!(addr.is_hostname());
    let addr = Address::from_str("10.0.23").expect("Unable to parse Address");
    assert!(addr.is_hostname());
    let addr = Address::from_str("2e3:4::10.0.23").expect("Unable to parse Address");
    assert!(addr.is_hostname());

    // IPv6
    let ipv6 = Ipv6Addr::from_str("4242::9").expect("Unable to parse IPv6");
    let addr = Address::from_str("4242::9").expect("Unable to parse Address");
    assert!(matches!(addr, Address::Ipv6(_)));
    assert_eq!(addr, Address::Ipv6(ipv6));

    // Wrong IPv6 should result in a hostname.
    let addr = Address::from_str("4242::9::5").expect("Unable to parse Address");
    assert!(addr.is_hostname());

    // Hostname
    let hostname = "www.torproject.org";
    let addr = Address::from_str(hostname).expect("Unable to parse Address");
    assert!(addr.is_hostname());
    assert_eq!(addr, Address::Hostname(hostname.to_string().into_bytes()));

    // Empty hostname
    let hostname = "";
    let addr = Address::from_str(hostname).expect("Unable to parse Address");
    assert!(addr.is_hostname());
    assert_eq!(addr, Address::Hostname(hostname.to_string().into_bytes()));

    // Too long hostname.
    let hostname = "a".repeat(256);
    let addr = Address::from_str(hostname.as_str());
    assert!(addr.is_err());
    assert_eq!(addr.err(), Some(Error::BadMessage("Hostname too long")));

    // Some Unicode emojis (go Gen-Z!).
    let hostname = "👍️👍️👍️";
    let addr = Address::from_str(hostname).expect("Unable to parse Address");
    assert!(addr.is_hostname());
    assert_eq!(addr, Address::Hostname(hostname.to_string().into_bytes()));

    // Address with nul byte. Not allowed.
    let hostname = "aaa\0aaa";
    let addr = Address::from_str(hostname);
    assert!(addr.is_err());
    assert_eq!(
        addr.err(),
        Some(Error::BadMessage("Nul byte not permitted"))
    );
}
