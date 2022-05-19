/*
    Copyright Michael Lodder. All Rights Reserved.
    SPDX-License-Identifier: Apache-2.0
*/
use super::invalid::*;
use super::valid::*;
use crate::{Feldman, FeldmanVerifier, Shamir};
use ff::PrimeField;
use p256::{NonZeroScalar, ProjectivePoint, Scalar, SecretKey};
use rand::rngs::OsRng;

#[test]
fn invalid_tests() {
    split_invalid_args::<Scalar, ProjectivePoint, 33>();
    combine_invalid::<Scalar, 33>();
}

#[test]
fn valid_tests() {
    combine_single::<Scalar, ProjectivePoint, 33>();
    combine_all::<Scalar, ProjectivePoint, 33>();
}

#[test]
fn key_tests() {
    let mut osrng = OsRng::default();
    let sk = SecretKey::random(&mut osrng);
    let nzs = sk.to_secret_scalar();
    let res = Shamir::<2, 3>::split_secret::<Scalar, OsRng, 33>(*nzs.as_ref(), &mut osrng);
    assert!(res.is_ok());
    let shares = res.unwrap();
    let res = Shamir::<2, 3>::combine_shares::<Scalar, 33>(&shares);
    assert!(res.is_ok());
    let scalar = res.unwrap();
    let nzs_dup = NonZeroScalar::from_repr(scalar.to_repr()).unwrap();
    let sk_dup = SecretKey::from(nzs_dup);
    assert_eq!(sk_dup.to_bytes(), sk.to_bytes());
}

#[test]
fn verifier_serde_test() {
    let mut osrng = OsRng::default();
    let sk = SecretKey::random(&mut osrng);
    let nzs = sk.to_secret_scalar();
    let res = Feldman::<2, 3>::split_secret::<Scalar, ProjectivePoint, OsRng, 33>(
        *nzs.as_ref(),
        None,
        &mut osrng,
    );
    assert!(res.is_ok());
    let (shares, verifier) = res.unwrap();
    for s in &shares {
        assert!(verifier.verify(s));
    }
    let res = serde_cbor::to_vec(&verifier);
    assert!(res.is_ok());
    let v_bytes = res.unwrap();
    let res = serde_cbor::from_slice::<FeldmanVerifier<Scalar, ProjectivePoint, 2>>(&v_bytes);
    assert!(res.is_ok());
    let verifier2 = res.unwrap();
    assert_eq!(verifier.generator, verifier2.generator);

    let res = serde_json::to_string(&verifier);
    assert!(res.is_ok());
    let v_str = res.unwrap();
    let res = serde_json::from_str::<FeldmanVerifier<Scalar, ProjectivePoint, 2>>(&v_str);
    assert!(res.is_ok());
    let verifier2 = res.unwrap();
    assert_eq!(verifier.generator, verifier2.generator);

    let res = serde_bare::to_vec(&verifier);
    assert!(res.is_ok());
    let v_bytes = res.unwrap();
    let res = serde_bare::from_slice::<FeldmanVerifier<Scalar, ProjectivePoint, 2>>(&v_bytes);
    assert!(res.is_ok());
    let verifier2 = res.unwrap();
    assert_eq!(verifier.generator, verifier2.generator);
}
