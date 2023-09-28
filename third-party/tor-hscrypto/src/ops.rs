//! Mid-level cryptographic operations used in the onion service protocol.

use tor_llcrypto::d::Sha3_256;
use tor_llcrypto::util::ct::CtByteArray;

use digest::Digest;

/// The length of the MAC returned by [`hs_mac`].
pub const HS_MAC_LEN: usize = 32;

/// Compute the lightweight MAC function used in the onion service protocol.
///
/// (See rend-spec-v3 section 0.3 `MAC`.)
///
/// This is not a great MAC; KMAC wasn't defined at the time that the HSv3
/// design was written. Please don't use this MAC in new protocols.
pub fn hs_mac(key: &[u8], msg: &[u8]) -> CtByteArray<HS_MAC_LEN> {
    // rend-spec-v3 says: "Instantiate H with SHA3-256... Instantiate MAC(key=k,
    // message=m) with H(k_len | k | m), where k_len is htonll(len(k))."

    let mut hasher = Sha3_256::new();
    let klen = key.len() as u64;
    hasher.update(klen.to_be_bytes());
    hasher.update(key);
    hasher.update(msg);
    let a: [u8; HS_MAC_LEN] = hasher.finalize().into();
    a.into()
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
    use hex_literal::hex;

    /// Helper: just call Sha3_256 and return the result as a CtByteArray.
    fn d(s: &[u8]) -> CtByteArray<32> {
        let a: [u8; 32] = Sha3_256::digest(s).into();
        a.into()
    }

    #[test]
    fn mac_from_definition() {
        assert_eq!(hs_mac(b"", b""), d(&[0; 8]));
        assert_eq!(
            hs_mac(b"hello", b"world"),
            d(b"\0\0\0\0\0\0\0\x05helloworld")
        );
        assert_eq!(
            hs_mac(b"helloworl", b"d"),
            d(b"\0\0\0\0\0\0\0\x09helloworld")
        );
    }

    #[test]
    fn mac_testvec() {
        // From C Tor; originally generated in Python.
        let msg = b"i am in a library somewhere using my computer";
        let key = b"i'm from the past talking to the future.";

        assert_eq!(
            hs_mac(key, msg).as_ref(),
            &hex!("753fba6d87d49497238a512a3772dd291e55f7d1cd332c9fb5c967c7a10a13ca")
        );
    }
}
