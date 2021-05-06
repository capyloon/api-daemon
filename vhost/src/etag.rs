// Build ETag values for File and ZipFile structs.
// This is done using cheap operations that don't
// require reading the file content.

use blake2::{Blake2s, Digest};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use zip::read::ZipFile;

pub struct Etag {}

impl Etag {
    // Builds the Etag with a Blake2 hash.
    // https://en.wikipedia.org/wiki/BLAKE_(hash_function)#BLAKE2
    pub fn for_file(mut file: &File) -> String {
        let mut hasher = Blake2s::new();
        let mut buffer = Vec::new();
        if file.read_to_end(&mut buffer).is_ok() {
            let _ = file.seek(SeekFrom::Start(0));
            hasher.update(&buffer);
            let res = hasher.finalize();
            format!("W/\"{:x}\"", res)
        } else {
            String::new()
        }
    }

    // Builds the Etag from the original file crc value and the size
    // of the zip entry.
    pub fn for_zip(file: &ZipFile) -> String {
        format!("W/\"{}-{}\"", file.crc32(), file.size())
    }
}
