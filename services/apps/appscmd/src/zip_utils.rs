// Zip file building.
// Credit to https://github.com/mvdnes/zip-rs/blob/master/examples/write_dir.rs

use log::debug;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::Path;
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};
use zip::result::ZipError;
use zip::write::FileOptions;
use zip::ZipWriter;

#[derive(Error, Debug)]
pub(crate) enum ZipperError {
    #[error("Zip Error {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("StripPrefix Error {0}")]
    StripPrefix(#[from] std::path::StripPrefixError),
    #[error("I/O Error {0}")]
    Io(#[from] std::io::Error),
}

type ZipperResult = Result<(), ZipperError>;

fn zip_dir<T>(it: &mut dyn Iterator<Item = DirEntry>, prefix: &str, writer: T) -> ZipperResult
where
    T: Write + Seek,
{
    let mut zip = ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix))?;

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            debug!("adding file {:?} as {:?} ...", path, name);
            zip.start_file(name.to_string_lossy(), options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            debug!("adding dir {:?} as {:?} ...", path, name);
            zip.add_directory(name.to_string_lossy(), options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

pub(crate) fn create_zip_for_dir(src_dir: &str, dst_file: &str) -> ZipperResult {
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound.into());
    }

    let path = Path::new(dst_file);
    let file = File::create(&path)?;

    let walkdir = WalkDir::new(src_dir.to_string());
    let it = walkdir.into_iter();

    zip_dir(&mut it.filter_map(|e| e.ok()), src_dir, file)?;

    Ok(())
}
