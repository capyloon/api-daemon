//! Relay cell cryptography
//!
//! The Tor protocol centers around "RELAY cells", which are
//! transmitted through the network along circuits.  The client that
//! creates a circuit shares two different set of keys and state with
//! each of the relays on the circuit: one for "outbound" traffic, and
//! one for "inbound" traffic.
//!

use crate::{Error, Result};
use tor_cell::chancell::RawCellBody;
use tor_error::internal;

use generic_array::GenericArray;

/// Type for the body of a relay cell.
#[derive(Clone)]
pub(crate) struct RelayCellBody(RawCellBody);

impl From<RawCellBody> for RelayCellBody {
    fn from(body: RawCellBody) -> Self {
        RelayCellBody(body)
    }
}
impl From<RelayCellBody> for RawCellBody {
    fn from(cell: RelayCellBody) -> Self {
        cell.0
    }
}
impl AsRef<[u8]> for RelayCellBody {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}
impl AsMut<[u8]> for RelayCellBody {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0[..]
    }
}

/// Represents the ability for a circuit crypto state to be initialized
/// from a given seed.
pub(crate) trait CryptInit: Sized {
    /// Return the number of bytes that this state will require.
    fn seed_len() -> usize;
    /// Construct this state from a seed of the appropriate length.
    fn initialize(seed: &[u8]) -> Result<Self>;
    /// Initialize this object from a key generator.
    fn construct<K: super::handshake::KeyGenerator>(keygen: K) -> Result<Self> {
        let seed = keygen.expand(Self::seed_len())?;
        Self::initialize(&seed[..])
    }
}

/// A paired object containing an inbound client layer and an outbound
/// client layer.
///
/// TODO: Maybe we should fold this into CryptInit.
pub(crate) trait ClientLayer<F, B>
where
    F: OutboundClientLayer,
    B: InboundClientLayer,
{
    /// Consume this ClientLayer and return a paired forward and reverse
    /// crypto layer.
    fn split(self) -> (F, B);
}

/// Represents a relay's view of the crypto state on a given circuit.
pub(crate) trait RelayCrypt {
    /// Prepare a RelayCellBody to be sent towards the client.
    fn originate(&mut self, cell: &mut RelayCellBody);
    /// Encrypt a RelayCellBody that is moving towards the client.
    fn encrypt_inbound(&mut self, cell: &mut RelayCellBody);
    /// Decrypt a RelayCellBody that is moving towards the client.
    ///
    /// Return true if it is addressed to us.
    fn decrypt_outbound(&mut self, cell: &mut RelayCellBody) -> bool;
}

/// A client's view of the crypto state shared with a single relay, as
/// used for outbound cells.
pub(crate) trait OutboundClientLayer {
    /// Prepare a RelayCellBody to be sent to the relay at this layer, and
    /// encrypt it.
    ///
    /// Return the authentication tag.
    fn originate_for(&mut self, cell: &mut RelayCellBody) -> &[u8];
    /// Encrypt a RelayCellBody to be decrypted by this layer.
    fn encrypt_outbound(&mut self, cell: &mut RelayCellBody);
}

/// A client's view of the crypto state shared with a single relay, as
/// used for inbound cells.
pub(crate) trait InboundClientLayer {
    /// Decrypt a CellBody that passed through this layer.
    ///
    /// Return an authentication tag if this layer is the originator.
    fn decrypt_inbound(&mut self, cell: &mut RelayCellBody) -> Option<&[u8]>;
}

/// Type to store hop indices on a circuit.
///
/// Hop indices are zero-based: "0" denotes the first hop on the circuit.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) struct HopNum(u8);

impl From<HopNum> for u8 {
    fn from(hop: HopNum) -> u8 {
        hop.0
    }
}

impl From<u8> for HopNum {
    fn from(v: u8) -> HopNum {
        HopNum(v)
    }
}

impl From<HopNum> for usize {
    fn from(hop: HopNum) -> usize {
        hop.0 as usize
    }
}

impl std::fmt::Display for HopNum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

/// A client's view of the cryptographic state for an entire
/// constructed circuit, as used for sending cells.
pub(crate) struct OutboundClientCrypt {
    /// Vector of layers, one for each hop on the circuit, ordered from the
    /// closest hop to the farthest.
    layers: Vec<Box<dyn OutboundClientLayer + Send>>,
}

/// A client's view of the cryptographic state for an entire
/// constructed circuit, as used for receiving cells.
pub(crate) struct InboundClientCrypt {
    /// Vector of layers, one for each hop on the circuit, ordered from the
    /// closest hop to the farthest.
    layers: Vec<Box<dyn InboundClientLayer + Send>>,
}

impl OutboundClientCrypt {
    /// Return a new (empty) OutboundClientCrypt.
    pub(crate) fn new() -> Self {
        OutboundClientCrypt { layers: Vec::new() }
    }
    /// Prepare a cell body to sent away from the client.
    ///
    /// The cell is prepared for the `hop`th hop, and then encrypted with
    /// the appropriate keys.
    ///
    /// On success, returns a reference to tag that should be expected
    /// for an authenticated SENDME sent in response to this cell.
    pub(crate) fn encrypt(&mut self, cell: &mut RelayCellBody, hop: HopNum) -> Result<&[u8; 20]> {
        let hop: usize = hop.into();
        if hop >= self.layers.len() {
            return Err(Error::NoSuchHop);
        }

        let mut layers = self.layers.iter_mut().take(hop + 1).rev();
        let first_layer = layers.next().ok_or(Error::NoSuchHop)?;
        let tag = first_layer.originate_for(cell);
        for layer in layers {
            layer.encrypt_outbound(cell);
        }
        Ok(tag.try_into().expect("wrong SENDME digest size"))
    }

    /// Add a new layer to this OutboundClientCrypt
    pub(crate) fn add_layer(&mut self, layer: Box<dyn OutboundClientLayer + Send>) {
        assert!(self.layers.len() < std::u8::MAX as usize);
        self.layers.push(layer);
    }

    /// Return the number of layers configured on this OutboundClientCrypt.
    pub(crate) fn n_layers(&self) -> usize {
        self.layers.len()
    }
}

impl InboundClientCrypt {
    /// Return a new (empty) InboundClientCrypt.
    pub(crate) fn new() -> Self {
        InboundClientCrypt { layers: Vec::new() }
    }
    /// Decrypt an incoming cell that is coming to the client.
    ///
    /// On success, return which hop was the originator of the cell.
    // TODO(nickm): Use a real type for the tag, not just `&[u8]`.
    pub(crate) fn decrypt(&mut self, cell: &mut RelayCellBody) -> Result<(HopNum, &[u8])> {
        for (hopnum, layer) in self.layers.iter_mut().enumerate() {
            if let Some(tag) = layer.decrypt_inbound(cell) {
                assert!(hopnum <= std::u8::MAX as usize);
                return Ok(((hopnum as u8).into(), tag));
            }
        }
        Err(Error::BadCellAuth)
    }
    /// Add a new layer to this InboundClientCrypt
    pub(crate) fn add_layer(&mut self, layer: Box<dyn InboundClientLayer + Send>) {
        assert!(self.layers.len() < std::u8::MAX as usize);
        self.layers.push(layer);
    }

    /// Return the number of layers configured on this InboundClientCrypt.
    ///
    /// TODO: use HopNum
    #[allow(dead_code)]
    pub(crate) fn n_layers(&self) -> usize {
        self.layers.len()
    }
}

/// Standard Tor relay crypto, as instantiated for RELAY cells.
pub(crate) type Tor1RelayCrypto =
    tor1::CryptStatePair<tor_llcrypto::cipher::aes::Aes128Ctr, tor_llcrypto::d::Sha1>;

/// Incomplete untested implementation of Tor's current cell crypto.
pub(crate) mod tor1 {
    use super::*;
    use cipher::{KeyIvInit, StreamCipher};
    use digest::Digest;
    use typenum::Unsigned;

    /// A CryptState is part of a RelayCrypt or a ClientLayer.
    ///
    /// It is parameterized on a stream cipher and a digest type: most
    /// circuits will use AES-128-CTR and SHA1, but v3 onion services
    /// use AES-256-CTR and SHA-3.
    pub(crate) struct CryptState<SC: StreamCipher, D: Digest + Clone> {
        /// Stream cipher for en/decrypting cell bodies.
        cipher: SC,
        /// Digest for authenticating cells to/from this hop.
        digest: D,
        /// Most recent digest value generated by this crypto.
        last_digest_val: GenericArray<u8, D::OutputSize>,
    }

    /// A pair of CryptStates, one for the forward (away from client)
    /// direction, and one for the reverse (towards client) direction.
    pub(crate) struct CryptStatePair<SC: StreamCipher, D: Digest + Clone> {
        /// State for en/decrypting cells sent away from the client.
        fwd: CryptState<SC, D>,
        /// State for en/decrypting cells sent towards the client.
        back: CryptState<SC, D>,
    }

    impl<SC: StreamCipher + KeyIvInit, D: Digest + Clone> CryptInit for CryptStatePair<SC, D> {
        fn seed_len() -> usize {
            SC::KeySize::to_usize() * 2 + D::OutputSize::to_usize() * 2
        }
        fn initialize(seed: &[u8]) -> Result<Self> {
            if seed.len() != Self::seed_len() {
                return Err(Error::from(internal!(
                    "seed length {} was invalid",
                    seed.len()
                )));
            }
            let keylen = SC::KeySize::to_usize();
            let dlen = D::OutputSize::to_usize();
            let fdinit = &seed[0..dlen];
            let bdinit = &seed[dlen..dlen * 2];
            let fckey = &seed[dlen * 2..dlen * 2 + keylen];
            let bckey = &seed[dlen * 2 + keylen..dlen * 2 + keylen * 2];
            let fwd = CryptState {
                cipher: SC::new(fckey.try_into().expect("Wrong length"), &Default::default()),
                digest: D::new().chain_update(fdinit),
                last_digest_val: GenericArray::default(),
            };
            let back = CryptState {
                cipher: SC::new(bckey.try_into().expect("Wrong length"), &Default::default()),
                digest: D::new().chain_update(bdinit),
                last_digest_val: GenericArray::default(),
            };
            Ok(CryptStatePair { fwd, back })
        }
    }

    impl<SC, D> ClientLayer<CryptState<SC, D>, CryptState<SC, D>> for CryptStatePair<SC, D>
    where
        SC: StreamCipher,
        D: Digest + Clone,
    {
        fn split(self) -> (CryptState<SC, D>, CryptState<SC, D>) {
            (self.fwd, self.back)
        }
    }

    impl<SC: StreamCipher, D: Digest + Clone> RelayCrypt for CryptStatePair<SC, D> {
        fn originate(&mut self, cell: &mut RelayCellBody) {
            let mut d_ignored = GenericArray::default();
            cell.set_digest(&mut self.back.digest, &mut d_ignored);
        }
        fn encrypt_inbound(&mut self, cell: &mut RelayCellBody) {
            self.back.cipher.apply_keystream(cell.as_mut());
        }
        fn decrypt_outbound(&mut self, cell: &mut RelayCellBody) -> bool {
            self.fwd.cipher.apply_keystream(cell.as_mut());
            let mut d_ignored = GenericArray::default();
            cell.recognized(&mut self.fwd.digest, &mut d_ignored)
        }
    }

    impl<SC: StreamCipher, D: Digest + Clone> OutboundClientLayer for CryptState<SC, D> {
        fn originate_for(&mut self, cell: &mut RelayCellBody) -> &[u8] {
            cell.set_digest(&mut self.digest, &mut self.last_digest_val);
            self.encrypt_outbound(cell);
            &self.last_digest_val
        }
        fn encrypt_outbound(&mut self, cell: &mut RelayCellBody) {
            self.cipher.apply_keystream(&mut cell.0[..]);
        }
    }

    impl<SC: StreamCipher, D: Digest + Clone> InboundClientLayer for CryptState<SC, D> {
        fn decrypt_inbound(&mut self, cell: &mut RelayCellBody) -> Option<&[u8]> {
            self.cipher.apply_keystream(&mut cell.0[..]);
            if cell.recognized(&mut self.digest, &mut self.last_digest_val) {
                Some(&self.last_digest_val)
            } else {
                None
            }
        }
    }

    impl RelayCellBody {
        /// Prepare a cell body by setting its digest and recognized field.
        fn set_digest<D: Digest + Clone>(
            &mut self,
            d: &mut D,
            used_digest: &mut GenericArray<u8, D::OutputSize>,
        ) {
            self.0[1] = 0;
            self.0[2] = 0;
            self.0[5] = 0;
            self.0[6] = 0;
            self.0[7] = 0;
            self.0[8] = 0;

            d.update(&self.0[..]);
            // TODO(nickm) can we avoid this clone?  Probably not.
            *used_digest = d.clone().finalize();
            self.0[5..9].copy_from_slice(&used_digest[0..4]);
        }
        /// Check a cell to see whether its recognized field is set.
        fn recognized<D: Digest + Clone>(
            &self,
            d: &mut D,
            rcvd: &mut GenericArray<u8, D::OutputSize>,
        ) -> bool {
            use crate::util::ct;
            use arrayref::array_ref;

            // Validate 'Recognized' field
            let recognized = u16::from_be_bytes(*array_ref![self.0, 1, 2]);
            if recognized != 0 {
                return false;
            }

            // Now also validate the 'Digest' field:

            let mut dtmp = d.clone();
            // Add bytes up to the 'Digest' field
            dtmp.update(&self.0[..5]);
            // Add zeroes where the 'Digest' field is
            dtmp.update([0_u8; 4]);
            // Add the rest of the bytes
            dtmp.update(&self.0[9..]);
            // Clone the digest before finalize destroys it because we will use
            // it in the future
            let dtmp_clone = dtmp.clone();
            let result = dtmp.finalize();

            if ct::bytes_eq(&self.0[5..9], &result[0..4]) {
                // Copy useful things out of this cell (we keep running digest)
                *d = dtmp_clone;
                *rcvd = result;
                return true;
            }

            false
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use rand::RngCore;
    use tor_basic_utils::test_rng::testing_rng;
    use tor_bytes::SecretBuf;

    fn add_layers(
        cc_out: &mut OutboundClientCrypt,
        cc_in: &mut InboundClientCrypt,
        pair: Tor1RelayCrypto,
    ) {
        let (outbound, inbound) = pair.split();
        cc_out.add_layer(Box::new(outbound));
        cc_in.add_layer(Box::new(inbound));
    }

    #[test]
    fn roundtrip() {
        // Take canned keys and make sure we can do crypto correctly.
        use crate::crypto::handshake::ShakeKeyGenerator as KGen;
        fn s(seed: &[u8]) -> SecretBuf {
            seed.to_vec().into()
        }

        let seed1 = s(b"hidden we are free");
        let seed2 = s(b"free to speak, to free ourselves");
        let seed3 = s(b"free to hide no more");

        let mut cc_out = OutboundClientCrypt::new();
        let mut cc_in = InboundClientCrypt::new();
        let pair = Tor1RelayCrypto::construct(KGen::new(seed1.clone())).unwrap();
        add_layers(&mut cc_out, &mut cc_in, pair);
        let pair = Tor1RelayCrypto::construct(KGen::new(seed2.clone())).unwrap();
        add_layers(&mut cc_out, &mut cc_in, pair);
        let pair = Tor1RelayCrypto::construct(KGen::new(seed3.clone())).unwrap();
        add_layers(&mut cc_out, &mut cc_in, pair);

        assert_eq!(cc_in.n_layers(), 3);
        assert_eq!(cc_out.n_layers(), 3);

        let mut r1 = Tor1RelayCrypto::construct(KGen::new(seed1)).unwrap();
        let mut r2 = Tor1RelayCrypto::construct(KGen::new(seed2)).unwrap();
        let mut r3 = Tor1RelayCrypto::construct(KGen::new(seed3)).unwrap();

        let mut rng = testing_rng();
        for _ in 1..300 {
            // outbound cell
            let mut cell = [0_u8; 509];
            let mut cell_orig = [0_u8; 509];
            rng.fill_bytes(&mut cell_orig);
            cell.copy_from_slice(&cell_orig);
            let mut cell = cell.into();
            let _tag = cc_out.encrypt(&mut cell, 2.into()).unwrap();
            assert_ne!(&cell.as_ref()[9..], &cell_orig.as_ref()[9..]);
            assert!(!r1.decrypt_outbound(&mut cell));
            assert!(!r2.decrypt_outbound(&mut cell));
            assert!(r3.decrypt_outbound(&mut cell));

            assert_eq!(&cell.as_ref()[9..], &cell_orig.as_ref()[9..]);

            // inbound cell
            let mut cell = [0_u8; 509];
            let mut cell_orig = [0_u8; 509];
            rng.fill_bytes(&mut cell_orig);
            cell.copy_from_slice(&cell_orig);
            let mut cell = cell.into();

            r3.originate(&mut cell);
            r3.encrypt_inbound(&mut cell);
            r2.encrypt_inbound(&mut cell);
            r1.encrypt_inbound(&mut cell);
            let (layer, _tag) = cc_in.decrypt(&mut cell).unwrap();
            assert_eq!(layer, 2.into());
            assert_eq!(&cell.as_ref()[9..], &cell_orig.as_ref()[9..]);

            // TODO: Test tag somehow.
        }

        // Try a failure: sending a cell to a nonexistent hop.
        {
            let mut cell = [0_u8; 509].into();
            let err = cc_out.encrypt(&mut cell, 10.into());
            assert!(matches!(err, Err(Error::NoSuchHop)));
        }

        // Try a failure: A junk cell with no correct auth from any layer.
        {
            let mut cell = [0_u8; 509].into();
            let err = cc_in.decrypt(&mut cell);
            assert!(matches!(err, Err(Error::BadCellAuth)));
        }
    }

    // From tor's test_relaycrypt.c

    #[test]
    fn testvec() {
        use digest::XofReader;
        use digest::{ExtendableOutput, Update};

        const K1: &[u8; 72] =
            b"    'My public key is in this signed x509 object', said Tom assertively.";
        const K2: &[u8; 72] =
            b"'Let's chart the pedal phlanges in the tomb', said Tom cryptographically";
        const K3: &[u8; 72] =
            b"     'Segmentation fault bugs don't _just happen_', said Tom seethingly.";

        const SEED: &[u8;108] = b"'You mean to tell me that there's a version of Sha-3 with no limit on the output length?', said Tom shakily.";

        // These test vectors were generated from Tor.
        let data: &[(usize, &str)] = &include!("../../testdata/cell_crypt.rs");

        let mut cc_out = OutboundClientCrypt::new();
        let mut cc_in = InboundClientCrypt::new();
        let pair = Tor1RelayCrypto::initialize(&K1[..]).unwrap();
        add_layers(&mut cc_out, &mut cc_in, pair);
        let pair = Tor1RelayCrypto::initialize(&K2[..]).unwrap();
        add_layers(&mut cc_out, &mut cc_in, pair);
        let pair = Tor1RelayCrypto::initialize(&K3[..]).unwrap();
        add_layers(&mut cc_out, &mut cc_in, pair);

        let mut xof = tor_llcrypto::d::Shake256::default();
        xof.update(&SEED[..]);
        let mut stream = xof.finalize_xof();

        let mut j = 0;
        for cellno in 0..51 {
            let mut body = [0_u8; 509];
            body[0] = 2; // command: data.
            body[4] = 1; // streamid: 1.
            body[9] = 1; // length: 498
            body[10] = 242;
            stream.read(&mut body[11..]);

            let mut cell = body.into();
            let _ = cc_out.encrypt(&mut cell, 2.into());

            if cellno == data[j].0 {
                let expected = hex::decode(data[j].1).unwrap();
                assert_eq!(cell.as_ref(), &expected[..]);
                j += 1;
            }
        }
    }
}
