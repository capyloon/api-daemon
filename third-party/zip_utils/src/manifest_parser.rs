//! manifest.mf parser.
//! The format is:
//!
//! Manifest-Version: 1.0
//!
//! Name: index.html
//! Digest-Algorithms: MD5 SHA1 SHA256
//! MD5-Digest: iUWtv5hMIDJgcxuch3MVnQ==
//! SHA1-Digest: HyEG55l3oKhy/9n12J9pdXxJldo=
//! SHA256-Digest: MibzowBZALsAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
//!
//! ... other file entries

use std::io::{BufRead, Seek, SeekFrom};

// We hardcode support for sha1 and sha256 only.
#[derive(Clone)]
pub struct ManifestEntry {
    pub name: String,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
}

pub struct Manifest {
    pub version: String,
    pub entries: Vec<ManifestEntry>,
}

fn parse_new_line<B: BufRead + Seek>(cur: &mut B) -> Result<(String, String), ()> {
    let mut buf = String::new();
    let mut buf_all = String::new();
    let mut parse_name = false;

    let _ = cur.read_line(&mut buf).map_err(|_| ())?;
    let mut current = cur.seek(SeekFrom::Current(0)).map_err(|_| ())? as i64;

    // Signing tool insert \n at length of 73 for file name
    // The following line starts with extra space
    // If it's not, we set cursor back to current line for next read
    // The max file name lenth is 138
    loop {
        // File name max length
        if buf_all.len() > 255 {
            break;
        }
        if buf.len() > 72 && buf.starts_with("Name: ") {
            parse_name = true;
            buf_all.push_str(&buf[..buf.len() - 1]);
        } else if buf.starts_with(' ') && parse_name {
            // The following line starts with " "
            buf_all.push_str(&buf[1..buf.len() - 1]);
            break;
        } else if parse_name {
            // Special case for 73 characters: "Name: " + 66
            // We pre-read new line to know if the file name
            // longer than 66. If not, we set cursor back for next round
            cur.seek(SeekFrom::Start(current as u64)).unwrap();
            break;
        } else {
            // Do nothing
            break;
        }
        buf.clear();
        current = cur.seek(SeekFrom::Current(0)).map_err(|_| ())? as i64;
        let _ = cur.read_line(&mut buf).map_err(|_| ())?;
    }
    if !buf_all.is_empty() {
        buf.clear();
        buf = buf_all;
    }
    let parts: Vec<&str> = buf.split(':').collect();
    if parts.len() != 2 {
        return Err(());
    }

    Ok((parts[0].trim().into(), parts[1].trim().into()))
}

pub fn read_manifest<B: BufRead + Seek>(mut cur: &mut B) -> Result<Manifest, ()> {
    // Parse the header.
    let header = parse_new_line(&mut cur)?;
    if header.0 != "Manifest-Version" {
        return Err(());
    }
    let version = header.1;

    let mut buf = String::new();
    let empty = cur.read_line(&mut buf);
    if empty.is_err() {
        return Ok(Manifest {
            version,
            entries: vec![],
        });
    }

    let mut entries: Vec<ManifestEntry> = vec![];

    macro_rules! get_line {
        ($name:expr) => {
            match parse_new_line(&mut cur) {
                Err(()) => {
                    // println!("No new line 2 for {}", $name);
                    return Ok(Manifest { version, entries });
                }
                Ok(val) => {
                    if val.0 != $name {
                        return Ok(Manifest { version, entries });
                    }
                    val.1
                }
            }
        };
    }

    // Each iteration reads an entry. If we reach EOF, return with
    // what we currently have.
    loop {
        // Get the name.
        let name = get_line!("Name");

        let mut entry = ManifestEntry {
            name,
            sha1: None,
            sha256: None,
        };

        // Get the algorithms and hashed values.
        for algo in get_line!("Digest-Algorithms").split(' ') {
            let value = get_line!(format!("{}-Digest", algo));
            match algo {
                "SHA1" => entry.sha1 = Some(value),
                "SHA256" => entry.sha256 = Some(value),
                _ => {}
            }
        }
        // Make sure we have at least a digest to check.
        // If not we don't add the entry to the list of files, which will
        // make the overall check fail because of the missing entry.
        if entry.sha1.is_some() || entry.sha256.is_some() {
            entries.push(entry);
        }

        // Read the empty line, or EOF.
        let mut buf = String::new();
        let empty = cur.read_line(&mut buf);
        if empty.is_err() {
            return Ok(Manifest { version, entries });
        }
    }
}

// Reads the zigbert.sf and extract the SHA1-Digest-Manifest
pub fn read_signature_manifest<B: BufRead + Seek>(mut cur: &mut B) -> Result<String, ()> {
    // Parse the header.
    let header = parse_new_line(&mut cur)?;
    if header.0 != "Signature-Version" {
        return Err(());
    }

    loop {
        let line = parse_new_line(&mut cur)?;
        if line.0 == "SHA1-Digest-Manifest" {
            return Ok(line.1);
        }
    }
}

#[test]
fn hash_manifest() {
    use std::fs::File;
    use std::io::{Cursor, Read};

    let mut file = File::open("test-fixtures/manifest.mf").unwrap();
    let mut content = Vec::new();
    let _ = file.read_to_end(&mut content).unwrap();
    let mut cursor = Cursor::new(content);

    let manifest = read_manifest(&mut cursor).unwrap();
    assert_eq!(manifest.version, "1.0");
    assert_eq!(manifest.entries.len(), 4);
    let entry = manifest.entries[2].clone();
    assert_eq!(entry.name, "style/icons/Default.png".to_string());
    assert_eq!(
        entry.sha1.unwrap(),
        "EEfSxfvlizAhlsfcnqZZwimA38A=".to_string()
    );
}

#[test]
fn hash_manifest_long_file_name() {
    use std::fs::File;
    use std::io::{Cursor, Read};

    let mut file = File::open("test-fixtures/long_name_manifest.mf").unwrap();
    let mut content = Vec::new();
    let _ = file.read_to_end(&mut content).unwrap();
    let mut cursor = Cursor::new(content);

    let manifest = read_manifest(&mut cursor).unwrap();
    assert_eq!(manifest.version, "1.0");
    assert_eq!(manifest.entries.len(), 5);
    let entry = manifest.entries[1].clone();
    assert_eq!(
        entry.name,
        "backendRoot~backend_content~frontendLoadMsgRange~migrations~page_stage2.js".to_string()
    );
    assert_eq!(
        entry.sha1.unwrap(),
        "390u+VjvTxVbaDN2bVFeUKFjCHo=".to_string()
    );
    assert_eq!(
        entry.sha256.unwrap(),
        "tuSksDWa95SrPjnCMJbkPis9JK2qdWAqeta6/ThkWOQ=".to_string()
    );
}

#[test]
fn hash_manifest_long_file_name_73() {
    use std::fs::File;
    use std::io::{Cursor, Read};

    let mut file = File::open("test-fixtures/manifest_73.mf").unwrap();
    let mut content = Vec::new();
    let _ = file.read_to_end(&mut content).unwrap();
    let mut cursor = Cursor::new(content);

    let manifest = read_manifest(&mut cursor).unwrap();
    assert_eq!(manifest.version, "1.0");
    assert_eq!(manifest.entries.len(), 5);
    let entry = manifest.entries[0].clone();
    assert_eq!(
        entry.name,
        "FileNamelengthis66xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".to_string()
    );
    assert_eq!(
        entry.sha1.unwrap(),
        "oKozUfffffUMKSFS6MLwgr7gFMQ=".to_string()
    );

    let entry = manifest.entries[1].clone();
    assert_eq!(entry.name, "index.js".to_string());
    assert_eq!(
        entry.sha1.unwrap(),
        "ltWPbFLx8ffPk9Cbh+ZAjVfLHjM=".to_string()
    );
}

#[test]
fn hash_manifest_longest_file_name() {
    use std::fs::File;
    use std::io::{Cursor, Read};

    let mut file = File::open("test-fixtures/manifest_longest.mf").unwrap();
    let mut content = Vec::new();
    let _ = file.read_to_end(&mut content).unwrap();
    let mut cursor = Cursor::new(content);

    let manifest = read_manifest(&mut cursor).unwrap();
    assert_eq!(manifest.version, "1.0");
    assert_eq!(manifest.entries.len(), 5);
    let entry = manifest.entries[2].clone();
    assert_eq!(
        entry.name,
        "Longest_file_name_is_138_as_signing_tool_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx138"
        .to_string()
    );
    assert_eq!(
        entry.sha1.unwrap(),
        "82ZW7CrVoMdttYe07aJJkHr7tPk=".to_string()
    );

    let entry = manifest.entries[3].clone();
    assert_eq!(
        entry.name,
        "File_name_length_is_67_just_one_more_line_xxxxxxxxxxxxxxxxxxxxxxxx3".to_string()
    );
    assert_eq!(
        entry.sha1.unwrap(),
        "oKozUUs4mGUMKSFS6MLwgr7gFMQ=".to_string()
    );

    let entry = manifest.entries[4].clone();
    assert_eq!(entry.name, "manifest.webapp".to_string());
    assert_eq!(
        entry.sha1.unwrap(),
        "Dq0S0487Lc0Z2ER9UUMzGMDP0dA=".to_string()
    );
}

#[test]
fn signature_manifest() {
    use std::fs::File;
    use std::io::{Cursor, Read};

    let mut file = File::open("test-fixtures/zigbert.sf").unwrap();
    let mut content = Vec::new();
    let _ = file.read_to_end(&mut content).unwrap();
    let mut cursor = Cursor::new(content);
    let hash = read_signature_manifest(&mut cursor).unwrap();
    assert_eq!(hash, "pu2xSwnv0PYXFJk9yjAaGBBcQ4I=");
}
