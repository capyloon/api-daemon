// Build ETag values for File and ZipFile structs.
// This is done using cheap operations that don't
// require reading the file content.

use std::fs::File;
use zip::read::ZipFile;

pub(crate) struct Etag {}

impl Etag {
    // Builds the Etag from the last modified time and the size
    // of the file.
    pub(crate) fn for_file(file: &File) -> String {
        if let Ok(metadata) = file.metadata() {
            match metadata.modified().map(|modified| {
                modified
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Modified is earlier than time::UNIX_EPOCH!")
            }) {
                Ok(modified) => format!(
                    "W/\"{}.{}-{}\"",
                    modified.as_secs(),
                    modified.subsec_nanos(),
                    metadata.len()
                ),
                _ => format!("W/\"{}\"", metadata.len()),
            }
        } else {
            String::new()
        }
    }

    // Builds the Etag from the original file crc value and the size
    // of the zip entry.
    pub(crate) fn for_zip(file: &ZipFile) -> String {
        format!("W/\"{}-{}\"", file.crc32(), file.size())
    }
}
