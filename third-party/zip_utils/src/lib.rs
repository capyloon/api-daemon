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
}

impl CertificateType {
    pub fn as_vec(&self) -> Vec<u8> {
        match &self {
            Self::Production => CERT_PROD.to_vec(),
            Self::Stage => CERT_DEV.to_vec(),
            Self::Test => CERT_TEST.to_vec(),
        }
    }
}

impl From<&str> for CertificateType {
    fn from(type_str: &str) -> Self {
        match type_str {
            "production" => Self::Production,
            "stage" => Self::Stage,
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
pub fn verify_zip<P: AsRef<Path>>(path: P, cert_type: &str) -> Result<(), ZipVerificationError> {
    let file = File::open(path).map_err(|_| ZipVerificationError::NoSuchFile)?;

    let mut archive = ZipArchive::new(file).map_err(|_| ZipVerificationError::InvalidZip)?;

    // 1. Verify the presence of mandatory files in META-INF. Any other file will be
    // referenced in the hash list in META-INF/manifest.mf
    for name in [
        "META-INF/zigbert.rsa",
        "META-INF/zigbert.sf",
        "META-INF/manifest.mf",
    ]
    .iter()
    {
        let _ = archive
            .by_name(name)
            .map_err(|_| ZipVerificationError::MissingMetaFile((*name).into()))?;
    }

    // 2. Get the parsed manifest.
    let mut content = Vec::new();
    {
        let mut file = archive.by_name("META-INF/manifest.mf").unwrap();
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
        // - the total number of files in the zip must be manifest.entries + 3 (special the META-INF ones)
        // - every file listed in the manifest must exist.
        // - their hashes must match.
        if manifest.entries.len() + 3 != archive.len() {
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
            let mut file = archive.by_name("META-INF/zigbert.sf").unwrap();
            let _ = file
                .read_to_end(&mut sf_content)
                .map_err(|_| ZipVerificationError::InvalidZip)?;
        }
        let mut sf_cursor = Cursor::new(sf_content);
        // 4. Use the META-INF/zigbert.sf to check the hash of META-INF/manifest.mf
        match read_signature_manifest(&mut sf_cursor) {
            Ok(manifest_hash) => {
                check_entry_digest(
                    &mut archive.by_name("META-INF/manifest.mf").unwrap(),
                    &digest::SHA1_FOR_LEGACY_USE_ONLY,
                    &manifest_hash,
                )?;
            }
            Err(_) => return Err(ZipVerificationError::InvalidManifest),
        }

        // 5. Check the signature of META-INF/zigbert.sf
        let mut rsa_file_buf: Vec<u8> = Vec::new();
        {
            let mut rsa_file = archive.by_name("META-INF/zigbert.rsa").unwrap();
            rsa_file
                .read_to_end(&mut rsa_file_buf)
                .map_err(|_| ZipVerificationError::InvalidZip)?;
        }

        let mut sf_file_buf: Vec<u8> = Vec::new();
        let mut sf_file = archive.by_name("META-INF/zigbert.sf").unwrap();
        sf_file
            .read_to_end(&mut sf_file_buf)
            .map_err(|_| ZipVerificationError::InvalidZip)?;
        let root_cert = CertificateType::from(cert_type).as_vec();
        sig_verification::verify(&rsa_file_buf, &sf_file_buf, &root_cert)?;
    } else {
        return Err(ZipVerificationError::InvalidManifest);
    }

    Ok(())
}

static CERT_TEST: &[u8] = include_bytes!("../service-center-test.crt");
static CERT_DEV: &[u8] = include_bytes!("../service-center-dev-public.crt");
static CERT_PROD: &[u8] = include_bytes!("../service-center-prod-public.crt");

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
    let result = verify_zip("test-fixtures/sample-signed.zip", "test");
    assert!(result.is_ok());

    // valid_sha256_zip
    let result = verify_zip("test-fixtures/app_sha256.zip", "test");
    assert!(result.is_ok());

    // valid api-daemon
    let result = verify_zip("test-fixtures/api-daemon-1.0.1.zip", "test");
    assert!(result.is_ok());

    let result = verify_zip("test-fixtures/api-daemon-1.1.2.zip", "test");
    assert!(result.is_ok());
}

#[test]
fn valid_zip_2certs_in_rsa() {
    // Stage server
    let result = verify_zip("test-fixtures/stage.zip", "stage");
    assert!(result.is_ok());

    // Production server
    let result = verify_zip("test-fixtures/prod.zip", "production");
    assert!(result.is_ok());
}

#[test]
fn test_verify_mismatch() {
    // Wrong cert
    let result = verify_zip("test-fixtures/prod.zip", "test");
    assert!(result.is_err());
}

#[test]
fn test_with_long_filename() {
    let result = verify_zip("test-fixtures/wa_stage.zip", "stage");
    assert!(result.is_ok());
}

#[test]
fn test_longest_filename() {
    let result = verify_zip("test-fixtures/longest_name.zip", "test");
    assert!(result.is_ok());
}
