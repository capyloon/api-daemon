//! Safe wrapper over mkstemp function from libc
//!
//! Usage example:
//!
//! ```rust
//! use std::io::Write;
//! extern crate mkstemp;
//! pub fn main() {
//!     // delete automatically when it goes out of scope
//!     let mut temp_file = mkstemp::TempFile::new("/tmp/testXXXXXX", true).unwrap();
//!     temp_file.write("test content".as_bytes()).unwrap();
//! }
//! ```

use std::io::{self, Read, Write};
use std::fs::{File, remove_file};
use std::os::unix::io::FromRawFd;
use std::ffi::CString;
use std::fmt::Arguments;

extern crate libc;

/// Temporary file
pub struct TempFile {
    // we use Option here as a trick to close the file in the drop trait
    file: Option<File>,
    path: String,
    auto_delete: bool
}

impl TempFile {
    /// Create temporary file
    ///
    /// * `template` - file template as described in mkstemp(3)<br/>
    /// * `auto_delete` - if true the file will be automatically deleted when it goes out of scope<br/>
    pub fn new(template: &str, auto_delete: bool) -> io::Result<TempFile> {
        let ptr = CString::new(template)?.into_raw();
        let fd = unsafe { libc::mkstemp(ptr) };
        let path = unsafe { CString::from_raw(ptr) };

        if fd < 0 {
            return Err(io::Error::last_os_error())
        }

        let file = unsafe { File::from_raw_fd(fd) };

        Ok(TempFile {
            file: Some(file),
            path: path.into_string().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?,
            auto_delete: auto_delete
        })
    }

    /// Return a reference to the actual file path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Return a mutable reference to inner file
    pub fn inner(&mut self) -> &mut File {
        self.file.as_mut().unwrap()
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        // close the file
        self.file = None;
        if self.auto_delete {
            let _ = remove_file(&self.path);
        }
    }
}

impl Read for TempFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner().read(buf)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.inner().read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.inner().read_to_string(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.inner().read_exact(buf)
    }
}

impl Write for TempFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner().flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.inner().write_all(buf)
    }

    fn write_fmt(&mut self, fmt: Arguments) -> io::Result<()> {
        self.inner().write_fmt(fmt)
    }
}
