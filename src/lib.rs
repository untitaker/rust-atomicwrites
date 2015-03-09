// DOCS

#![feature(path,io)]

extern crate tempdir;

use std::io;
use std::fs;
use std::borrow::Borrow;
use std::path;

use tempdir::TempDir;
pub use OverwriteBehavior::{AllowOverwrite, DisallowOverwrite};


#[derive(Copy)]
pub enum OverwriteBehavior {
    /// Overwrite files silently.
    AllowOverwrite,
    
    /// Don't overwrite files. `AtomicFile.write` will raise errors for such conditions only after
    /// you've already written your data.
    DisallowOverwrite
}

pub struct AtomicFile {
    path: path::PathBuf,
    overwrite: OverwriteBehavior,
    tmpdir: path::PathBuf
}


impl AtomicFile {
    pub fn new_with_tmpdir(path: &path::Path, overwrite: OverwriteBehavior, tmpdir: &path::Path) -> Self {
        AtomicFile {
            path: path.to_path_buf(),
            overwrite: overwrite,
            tmpdir: tmpdir.to_path_buf()
        }
    }

    fn commit(&self, tmppath: &path::Path) -> io::Result<()> {
        match self.overwrite {
            AllowOverwrite => replace_atomic(tmppath, self.path()),
            DisallowOverwrite => move_atomic(tmppath, self.path())
        }
    }

    /// Helper for writing to `path` in write-only mode.
    ///
    /// If `DisallowOverwrite` is given, errors will be returned from `self.write(...)` if the file
    /// exists.
    pub fn new(path: &path::Path, overwrite: OverwriteBehavior) -> Self {
        AtomicFile::new_with_tmpdir(path, overwrite, &path.parent().unwrap_or(&path))
    }

    /// Get the target filepath.
    pub fn path(&self) -> &path::Path { &self.path.borrow() }


    /// Open a temporary file, call `f` on it (which is supposed to write to it), then move the
    /// file atomically to `self.path`.
    pub fn write<F: FnMut(&mut fs::File) -> io::Result<()>>(&self, mut f: F) -> io::Result<()> {
        let tmpdir = match TempDir::new_in(
            &self.tmpdir,
            ".atomicwrite"
        ) {
            Ok(x) => x,
            Err(_) => return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to create a temporary directory.",
                None
            ))
        };

        let tmppath = tmpdir.path().join("tmpfile.tmp");
        {
            let mut tmpfile = try!(fs::File::create(&tmppath));
            try!(f(&mut tmpfile));
        };
        try!(self.commit(&tmppath));
        Ok(())
    }
}

#[cfg(unix)]
fn replace_atomic_impl(src: &path::Path, dst: &path::Path) -> io::Result<()> {
    fs::rename(src, dst)
}

#[cfg(unix)]
fn move_atomic_impl(src: &path::Path, dst: &path::Path) -> io::Result<()> {
    try!(fs::hard_link(src, dst));
    fs::remove_file(src)
}

/// Move `src` to `dst`. If `dst` exists, it will be silently overwritten.
///
/// Both paths must reside on the same filesystem for the operation to be atomic.
pub fn replace_atomic(src: &path::Path, dst: &path::Path) -> io::Result<()> {
    replace_atomic_impl(src, dst)
}

/// Move `src` to `dst`. An error will be returned if `dst` exists.
///
/// Both paths must reside on the same filesystem for the operation to be atomic.
pub fn move_atomic(src: &path::Path, dst: &path::Path) -> io::Result<()> {
    move_atomic_impl(src, dst)
}
