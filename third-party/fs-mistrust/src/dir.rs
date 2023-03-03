//! Implement a wrapper for access to the members of a directory whose status
//! we've checked.

use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::{walk::PathType, Error, Mistrust, Result, Verifier};

#[cfg(target_family = "unix")]
use std::os::unix::fs::OpenOptionsExt;

/// A directory whose access properties we have verified, along with accessor
/// functions to access members of that directory.
///
/// The accessor functions will enforce that whatever security properties we
/// checked on the the directory also apply to all of the members that we access
/// within the directory.
///
/// ## Limitations
///
/// Having a `CheckedDir` means only that, at the time it was created, we were
/// confident that no _untrusted_ user could access it inappropriately.  It is
/// still possible, after the `CheckedDir` is created, that a _trusted_ user can
/// alter its permissions, make its path point somewhere else, or so forth.
///
/// If this kind of time-of-use/time-of-check issue is unacceptable, you may
/// wish to look at other solutions, possibly involving `openat()` or related
/// APIs.
///
/// See also the crate-level [Limitations](crate#limitations) section.
#[derive(Debug, Clone)]
pub struct CheckedDir {
    /// The `Mistrust` object whose rules we apply to members of this directory.
    mistrust: Mistrust,
    /// The location of this directory, in its original form.
    location: PathBuf,
    /// The "readable_okay" flag that we used to create this CheckedDir.
    readable_okay: bool,
}

impl CheckedDir {
    /// Create a CheckedDir.
    pub(crate) fn new(verifier: &Verifier<'_>, path: &Path) -> Result<Self> {
        let mut mistrust = verifier.mistrust.clone();
        // Ignore the path that we already verified.  Since ignore_prefix
        // canonicalizes the path, we _will_ recheck the directory if it starts
        // pointing to a new canonical location.  That's probably a feature.
        //
        // TODO:
        //   * If `path` is a prefix of the original ignored path, this will
        //     make us ignore _less_.
        mistrust.ignore_prefix = crate::canonicalize_opt_prefix(&Some(Some(path.to_path_buf())))?;
        Ok(CheckedDir {
            mistrust,
            location: path.to_path_buf(),
            readable_okay: verifier.readable_okay,
        })
    }

    /// Construct a new directory within this CheckedDir, if it does not already
    /// exist.
    ///
    /// `path` must be a relative path to the new directory, containing no `..`
    /// components.
    pub fn make_directory<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        self.check_path(path)?;
        self.verifier().make_directory(self.location.join(path))
    }

    /// Open a file within this CheckedDir, using a set of [`OpenOptions`].
    ///
    /// `path` must be a relative path to the new directory, containing no `..`
    /// components.  We check, but do not create, the file's parent directories.
    /// We check the file's permissions after opening it.  If the file already
    /// exists, it must not be a symlink.
    ///
    /// If the file is created (and this is a unix-like operating system), we
    /// always create it with mode `600`, regardless of any mode options set in
    /// `options`.
    pub fn open<P: AsRef<Path>>(&self, path: P, options: &OpenOptions) -> Result<File> {
        let path = path.as_ref();
        self.check_path(path)?;
        let path = self.location.join(path);
        if let Some(parent) = path.parent() {
            self.verifier().check(parent)?;
        }

        #[allow(unused_mut)]
        let mut options = options.clone();

        #[cfg(target_family = "unix")]
        {
            // By default, create all files mode 600, no matter what
            // OpenOptions said.

            // TODO: Give some way to override this to 640 or 0644 if you
            //    really want to.
            options.mode(0o600);
            // Don't follow symlinks out of the secured directory.
            options.custom_flags(libc::O_NOFOLLOW);
        }

        let file = options
            .open(&path)
            .map_err(|e| Error::io(e, &path, "open file"))?;
        let meta = file.metadata().map_err(|e| Error::inspecting(e, &path))?;

        if let Some(error) = self
            .verifier()
            .check_one(path.as_path(), PathType::Content, &meta)
            .into_iter()
            .next()
        {
            Err(error)
        } else {
            Ok(file)
        }
    }

    /// Return a reference to this directory as a [`Path`].
    ///
    /// Note that this function lets you work with a broader collection of
    /// functions, including functions that might let you access or create a
    /// file that is accessible by non-trusted users.  Be careful!
    pub fn as_path(&self) -> &Path {
        self.location.as_path()
    }

    /// Return a new [`PathBuf`] containing this directory's path, with `path`
    /// appended to it.
    ///
    /// Return an error if `path` has any components that could take us outside
    /// of this directory.
    pub fn join<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        let path = path.as_ref();
        self.check_path(path)?;
        Ok(self.location.join(path))
    }

    /// Read the contents of the file at `path` within this directory, as a
    /// String, if possible.
    ///
    /// Return an error if `path` is absent, if its permissions are incorrect,
    /// if it has any components that could take us outside of this directory,
    /// or if its contents are not UTF-8.
    pub fn read_to_string<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let path = path.as_ref();
        let mut file = self.open(path, OpenOptions::new().read(true))?;
        let mut result = String::new();
        file.read_to_string(&mut result)
            .map_err(|e| Error::io(e, path, "read file"))?;
        Ok(result)
    }

    /// Read the contents of the file at `path` within this directory, as a
    /// vector of bytes, if possible.
    ///
    /// Return an error if `path` is absent, if its permissions are incorrect,
    /// or if it has any components that could take us outside of this
    /// directory.
    pub fn read<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>> {
        let path = path.as_ref();
        let mut file = self.open(path, OpenOptions::new().read(true))?;
        let mut result = Vec::new();
        file.read_to_end(&mut result)
            .map_err(|e| Error::io(e, path, "read file"))?;
        Ok(result)
    }

    /// Store `contents` into the file located at `path` within this directory.
    ///
    /// We won't write to `path` directly: instead, we'll write to a temporary
    /// file in the same directory as `path`, and then replace `path` with that
    /// temporary file if we were successful.  (This isn't truly atomic on all
    /// file systems, but it's closer than many alternatives.)
    ///
    /// # Limitations
    ///
    /// This function will clobber any existing files with the same name as
    /// `path` but with the extension `tmp`.  (That is, if you are writing to
    /// "foo.txt", it will replace "foo.tmp" in the same directory.)
    ///
    /// This function may give incorrect behavior if multiple threads or
    /// processes are writing to the same file at the same time: it is the
    /// programmer's responsibility to use appropriate locking to avoid this.
    pub fn write_and_replace<P: AsRef<Path>, C: AsRef<[u8]>>(
        &self,
        path: P,
        contents: C,
    ) -> Result<()> {
        let path = path.as_ref();
        self.check_path(path)?;

        let tmp_name = path.with_extension("tmp");
        let mut tmp_file = self.open(
            &tmp_name,
            OpenOptions::new().create(true).truncate(true).write(true),
        )?;

        // Write the data.
        tmp_file
            .write_all(contents.as_ref())
            .map_err(|e| Error::io(e, &tmp_name, "write to file"))?;
        // Flush and close.
        drop(tmp_file);

        // Replace the old file.
        std::fs::rename(self.location.join(tmp_name), self.location.join(path))
            .map_err(|e| Error::io(e, path, "replace file"))?;
        Ok(())
    }

    /// Helper: create a [`Verifier`] with the appropriate rules for this
    /// `CheckedDir`.
    fn verifier(&self) -> Verifier<'_> {
        let mut v = self.mistrust.verifier();
        if self.readable_okay {
            v = v.permit_readable();
        }
        v
    }

    /// Helper: Make sure that the path `p` is a relative path that can be
    /// guaranteed to stay within this directory.
    fn check_path(&self, p: &Path) -> Result<()> {
        use std::path::Component;
        // This check should be redundant, but let's be certain.
        if p.is_absolute() {
            return Err(Error::InvalidSubdirectory);
        }

        for component in p.components() {
            match component {
                Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                    return Err(Error::InvalidSubdirectory)
                }
                Component::CurDir | Component::Normal(_) => {}
            }
        }

        Ok(())
    }
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
    use crate::testing::Dir;
    use std::io::Write;

    #[test]
    fn easy_case() {
        let d = Dir::new();
        d.dir("a/b/c");
        d.dir("a/b/d");
        d.file("a/b/c/f1");
        d.file("a/b/c/f2");
        d.file("a/b/d/f3");

        d.chmod("a", 0o755);
        d.chmod("a/b", 0o700);
        d.chmod("a/b/c", 0o700);
        d.chmod("a/b/d", 0o777);
        d.chmod("a/b/c/f1", 0o600);
        d.chmod("a/b/c/f2", 0o666);
        d.chmod("a/b/d/f3", 0o600);

        let m = Mistrust::builder()
            .ignore_prefix(d.canonical_root())
            .build()
            .unwrap();

        let sd = m.verifier().secure_dir(d.path("a/b")).unwrap();

        // Try make_directory.
        sd.make_directory("c/sub1").unwrap();
        #[cfg(target_family = "unix")]
        {
            let e = sd.make_directory("d/sub2").unwrap_err();
            assert!(matches!(e, Error::BadPermission(..)));
        }

        // Try opening a file that exists.
        let f1 = sd.open("c/f1", OpenOptions::new().read(true)).unwrap();
        drop(f1);
        #[cfg(target_family = "unix")]
        {
            let e = sd.open("c/f2", OpenOptions::new().read(true)).unwrap_err();
            assert!(matches!(e, Error::BadPermission(..)));
            let e = sd.open("d/f3", OpenOptions::new().read(true)).unwrap_err();
            assert!(matches!(e, Error::BadPermission(..)));
        }

        // Try creating a file.
        let mut f3 = sd
            .open("c/f-new", OpenOptions::new().write(true).create(true))
            .unwrap();
        f3.write_all(b"Hello world").unwrap();
        drop(f3);

        #[cfg(target_family = "unix")]
        {
            let e = sd
                .open("d/f-new", OpenOptions::new().write(true).create(true))
                .unwrap_err();
            assert!(matches!(e, Error::BadPermission(..)));
        }
    }

    #[test]
    fn bad_paths() {
        let d = Dir::new();
        d.dir("a");
        d.chmod("a", 0o700);

        let m = Mistrust::builder()
            .ignore_prefix(d.canonical_root())
            .build()
            .unwrap();

        let sd = m.verifier().secure_dir(d.path("a")).unwrap();

        let e = sd.make_directory("hello/../world").unwrap_err();
        assert!(matches!(e, Error::InvalidSubdirectory));
        let e = sd.make_directory("/hello").unwrap_err();
        assert!(matches!(e, Error::InvalidSubdirectory));

        sd.make_directory("hello/world").unwrap();
    }

    #[test]
    fn read_and_write() {
        let d = Dir::new();
        d.dir("a");
        d.chmod("a", 0o700);
        let m = Mistrust::builder()
            .ignore_prefix(d.canonical_root())
            .build()
            .unwrap();

        let checked = m.verifier().secure_dir(d.path("a")).unwrap();

        // Simple case: write and read.
        checked
            .write_and_replace("foo.txt", "this is incredibly silly")
            .unwrap();

        let s1 = checked.read_to_string("foo.txt").unwrap();
        let s2 = checked.read("foo.txt").unwrap();
        assert_eq!(s1, "this is incredibly silly");
        assert_eq!(s1.as_bytes(), &s2[..]);

        // Trickier: write when the preferred temporary already has content.
        checked
            .open("bar.tmp", OpenOptions::new().create(true).write(true))
            .unwrap()
            .write_all("be the other guy".as_bytes())
            .unwrap();
        assert!(checked.join("bar.tmp").unwrap().exists());

        checked
            .write_and_replace("bar.txt", "its hard and nobody understands")
            .unwrap();

        // Temp file should be gone.
        assert!(!checked.join("bar.tmp").unwrap().exists());
        let s4 = checked.read_to_string("bar.txt").unwrap();
        assert_eq!(s4, "its hard and nobody understands");
    }
}
