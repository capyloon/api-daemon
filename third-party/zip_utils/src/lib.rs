//! A utility to check the validity of signed zip archives.

extern crate base64;
extern crate ring;
extern crate untrusted;
extern crate zip;

mod manifest_parser;
mod sig_verification;

use self::manifest_parser::*;
use ring::digest;
use simple_asn1::ASN1DecodeErr;
use simple_asn1::ASN1EncodeErr;
use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

#[derive(PartialEq, Eq)]
pub enum CertificateType {
    Production,
    Stage,
    Test,
    Ven1,
    Ven2,
}

impl CertificateType {
    pub fn as_vec(&self) -> Vec<u8> {
        match &self {
            Self::Production => CERT_PROD.to_vec(),
            Self::Stage => CERT_DEV.to_vec(),
            Self::Test => CERT_TEST.to_vec(),
            Self::Ven1 => CERT_VEN1.to_vec(),
            Self::Ven2 => CERT_VEN2.to_vec(),
        }
    }
}

impl From<&str> for CertificateType {
    fn from(type_str: &str) -> Self {
        match type_str {
            "production" => Self::Production,
            "stage" => Self::Stage,
            "ven1" => Self::Ven1,
            "ven2" => Self::Ven2,
            _ => Self::Test,
        }
    }
}

#[derive(Debug)]
pub enum ZipVerificationError {
    NoSuchFile,
    InvalidZip,
    MissingMetaFile(String),
    InvalidHash,
    InvalidSignature,
    InvalidFileList,
    InvalidManifest,
    InvalidRsaFileDecode(ASN1DecodeErr),
    InvalidRsaFileEncode(ASN1EncodeErr),
    DerefASN1Error,
    CertificateExpired,
    X509Signature(x509_signature::Error),
}

impl From<simple_asn1::ASN1DecodeErr> for ZipVerificationError {
    fn from(error: simple_asn1::ASN1DecodeErr) -> Self {
        ZipVerificationError::InvalidRsaFileDecode(error)
    }
}

impl From<simple_asn1::ASN1EncodeErr> for ZipVerificationError {
    fn from(error: simple_asn1::ASN1EncodeErr) -> Self {
        ZipVerificationError::InvalidRsaFileEncode(error)
    }
}

impl From<x509_signature::Error> for ZipVerificationError {
    fn from(error: x509_signature::Error) -> Self {
        ZipVerificationError::X509Signature(error)
    }
}

// We sign the app last, that means any already signed files/folder
// are treated as normal file by us.
// But one app could be only signed by vendors,
// so, the function tells the number of files to be excluded
fn get_number_excluded<R: Read + std::io::Seek>(
    folder_name: &str,
    archive: &mut ZipArchive<R>,
) -> usize {
    let ids_service = "META-INF/ids.json";
    if is_service_signing(folder_name) || archive.by_name(ids_service).is_err() {
        3
    } else {
        7
    }
}

// Verify the digest of a readable stream using the given algorithm and
// expected value.
fn check_entry_digest<R: Read>(
    input: &mut R,
    algorithm: &'static digest::Algorithm,
    expected: &str,
) -> Result<(), ZipVerificationError> {
    let mut context = digest::Context::new(algorithm);

    loop {
        let mut buffer = [0; 4096];
        let count = input
            .read(&mut buffer[..])
            .map_err(|_| ZipVerificationError::InvalidHash)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    // Convert the byte representation into the expected text format.
    let result = base64::encode(context.finish().as_ref());

    if result == expected {
        Ok(())
    } else {
        println!("Expected {} but got {}", expected, result);
        Err(ZipVerificationError::InvalidHash)
    }
}

// Verifies the hashes and signature of a zip file at the given path.
pub fn verify_zip<P: AsRef<Path>>(
    path: P,
    cert_type: &str,
    folder_name: &str,
) -> Result<String, ZipVerificationError> {
    let file = File::open(path).map_err(|_| ZipVerificationError::NoSuchFile)?;

    let mut archive = ZipArchive::new(file).map_err(|_| ZipVerificationError::InvalidZip)?;

    let meta_dir = meta_folder(folder_name);
    let zigbert_rsa = format!("{}/zigbert.rsa", meta_dir);
    let zigbert_sf = format!("{}/zigbert.sf", meta_dir);
    let manifest_mf = format!("{}/manifest.mf", meta_dir);

    // 1. Verify the presence of mandatory files in META-*. Any other file will be
    // referenced in the hash list in META-*/manifest.mf
    for name in [&zigbert_rsa, &zigbert_sf, &manifest_mf].iter() {
        let _ = archive
            .by_name(name)
            .map_err(|_| ZipVerificationError::MissingMetaFile((*name).into()))?;
    }

    // 2. Get the parsed manifest.
    let mut content = Vec::new();
    {
        let mut file = archive.by_name(&manifest_mf).unwrap();
        let _ = file
            .read_to_end(&mut content)
            .map_err(|_| ZipVerificationError::InvalidZip)?;
    }
    let mut cursor = Cursor::new(content);
    if let Ok(manifest) = read_manifest(&mut cursor) {
        if manifest.version != "1.0" {
            return Err(ZipVerificationError::InvalidManifest);
        }

        // 3. Check that the list of files in the manifest matches the list of files in the zip:
        // - every file listed in the manifest must exist.
        // - their hashes must match.
        let excluded = get_number_excluded(folder_name, &mut archive);
        if manifest.entries.len() + excluded != archive.len() {
            return Err(ZipVerificationError::InvalidFileList);
        }
        for entry in manifest.entries {
            match archive.by_name(&entry.name) {
                Err(_) => return Err(ZipVerificationError::InvalidFileList),
                Ok(mut zipentry) => {
                    if let Some(sha1) = entry.sha1 {
                        check_entry_digest(
                            &mut zipentry,
                            &digest::SHA1_FOR_LEGACY_USE_ONLY,
                            &sha1,
                        )?;
                    } else if let Some(sha256) = entry.sha256 {
                        check_entry_digest(&mut zipentry, &digest::SHA256, &sha256)?;
                    }
                }
            }
        }

        let mut sf_content = Vec::new();
        {
            let mut file = archive.by_name(&zigbert_sf).unwrap();
            let _ = file
                .read_to_end(&mut sf_content)
                .map_err(|_| ZipVerificationError::InvalidZip)?;
        }
        let mut sf_cursor = Cursor::new(sf_content);
        // 4. Use the META-*/zigbert.sf to check the hash of META-*/manifest.mf
        match read_signature_manifest(&mut sf_cursor) {
            Ok(manifest_hash) => {
                check_entry_digest(
                    &mut archive.by_name(&manifest_mf).unwrap(),
                    &digest::SHA1_FOR_LEGACY_USE_ONLY,
                    &manifest_hash,
                )?;
            }
            Err(_) => return Err(ZipVerificationError::InvalidManifest),
        }

        // 5. Check the signature of META-*/zigbert.sf
        let mut rsa_file_buf: Vec<u8> = Vec::new();
        {
            let mut rsa_file = archive.by_name(&zigbert_rsa).unwrap();
            rsa_file
                .read_to_end(&mut rsa_file_buf)
                .map_err(|_| ZipVerificationError::InvalidZip)?;
        }

        let mut sf_file_buf: Vec<u8> = Vec::new();
        {
            let mut sf_file = archive.by_name(&zigbert_sf).unwrap();
            sf_file
                .read_to_end(&mut sf_file_buf)
                .map_err(|_| ZipVerificationError::InvalidZip)?;
        }
        let root_cert = CertificateType::from(cert_type).as_vec();
        let fingerprint = sig_verification::verify(&rsa_file_buf, &sf_file_buf, &root_cert)?;

        return Ok(fingerprint);
    } else {
        return Err(ZipVerificationError::InvalidManifest);
    }
}

pub fn meta_folder(name: &str) -> String {
    let folder = name.to_uppercase();

    format!("META-{}", folder)
}

pub fn is_service_signing(name: &str) -> bool {
    meta_folder(name) == "META-INF"
}

pub fn get_fingerprint(cert_raw: &[u8]) -> Result<String, ZipVerificationError> {
    Ok(sig_verification::fingerprint(cert_raw)?)
}

pub fn get_public_key(cert_raw: &[u8]) -> Result<String, ZipVerificationError> {
    Ok(sig_verification::get_public_key(cert_raw)?)
}

static CERT_TEST: &[u8] = include_bytes!("../service-center-test.crt");
static CERT_DEV: &[u8] = include_bytes!("../service-center-dev-public.crt");
static CERT_PROD: &[u8] = include_bytes!("../service-center-prod-public.crt");
static CERT_VEN1: &[u8] = include_bytes!("../vendor1.crt");
static CERT_VEN2: &[u8] = include_bytes!("../vendor2.crt");

#[test]
fn test_get_cert_type() {
    let cert = CertificateType::from("production").as_vec();

    let mut root_cert: Vec<u8> = Vec::new();
    let mut root_cert_file = File::open("./service-center-prod-public.crt").unwrap();
    root_cert_file.read_to_end(&mut root_cert).unwrap();
    assert_eq!(cert, root_cert);

    {
        let cert = CertificateType::from("stage").as_vec();

        let mut root_cert: Vec<u8> = Vec::new();
        let mut root_cert_file = File::open("./service-center-dev-public.crt").unwrap();
        root_cert_file.read_to_end(&mut root_cert).unwrap();
        assert_eq!(cert, root_cert);
    }

    {
        let cert = CertificateType::from("test").as_vec();

        let mut root_cert: Vec<u8> = Vec::new();
        let mut root_cert_file = File::open("./service-center-test.crt").unwrap();
        root_cert_file.read_to_end(&mut root_cert).unwrap();
        assert_eq!(cert, root_cert);
    }
}

#[test]
fn valid_zip() {
    let result = verify_zip("test-fixtures/sample-signed.zip", "test", "INF");
    assert!(result.is_ok());

    // valid_sha256_zip
    let result = verify_zip("test-fixtures/app_sha256.zip", "test", "INF");
    assert!(result.is_ok());

    // valid api-daemon
    let result = verify_zip("test-fixtures/api-daemon-1.0.1.zip", "test", "INF");
    assert!(result.is_ok());

    let result = verify_zip("test-fixtures/api-daemon-1.1.2.zip", "test", "INF");
    assert!(result.is_ok());
}

#[test]
fn valid_zip_2certs_in_rsa() {
    // Stage server
    let result = verify_zip("test-fixtures/stage.zip", "stage", "INF");
    assert!(result.is_ok());

    // Production server
    let result = verify_zip("test-fixtures/prod.zip", "production", "INF");
    assert!(result.is_ok());
}

#[test]
fn test_verify_mismatch() {
    // Wrong cert
    let result = verify_zip("test-fixtures/prod.zip", "test", "INF");
    assert!(result.is_err());
}

#[test]
fn test_with_long_filename() {
    let result = verify_zip("test-fixtures/wa_stage.zip", "stage", "INF");
    assert!(result.is_ok());
}

#[test]
fn test_longest_filename() {
    let result = verify_zip("test-fixtures/longest_name.zip", "test", "INF");
    assert!(result.is_ok());
}

#[test]
fn test_ven1() {
    if let Err(err) = verify_zip("test-fixtures/ven1-sample.zip", "ven1", "ven") {
        println!("test ven1-sample.zip Err: {:?}", err);
        assert!(false);
    }
}

#[test]
fn test_ven() {
    if let Err(err) = verify_zip("test-fixtures/ven-sample.zip", "ven2", "ven") {
        println!("test ven-sample.zip Err: {:?}", err);
        assert!(false);
    }
}

#[test]
fn test_double() {
    if let Err(err) = verify_zip("test-fixtures/ven1-sample-double-signed.zip", "test", "INF") {
        println!("test ven1-sample-double-signed META-INF Err: {:?}", err);
        assert!(false);
    }

    if let Err(err) = verify_zip("test-fixtures/ven1-sample-double-signed.zip", "ven1", "ven") {
        println!("test ven1-sample-double-signed.zip META-INF Err: {:?}", err);
        assert!(false);
    }
}

#[test]
fn test_no_signature() {
    if let Err(err) = verify_zip("test-fixtures/hello.zip", "ven1", "ven1") {
        println!("test test_no_signature.zip  Err: {:?}", err);
        assert!(true);
    } else {
        assert!(false);
    }
}
