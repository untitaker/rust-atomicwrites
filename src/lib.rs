// DOCS

extern crate tempfile;

use std::io;
use std::fs;
use std::mem;
use std::borrow::Borrow;
use std::path;
use std::convert::AsRef;

use tempfile::NamedTempFile;

pub use OverwriteBehavior::{AllowOverwrite, DisallowOverwrite};


#[derive(Clone,Copy)]
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
    /// Helper for writing to `path` in write-only mode.
    ///
    /// If `DisallowOverwrite` is given, errors will be returned from `self.write(...)` if the file
    /// exists.
    pub fn new<P: AsRef<path::Path>>(path: P, overwrite: OverwriteBehavior) -> Self {
        let p = path.as_ref();
        AtomicFile::new_with_tmpdir(p, overwrite, p.parent().unwrap_or(p))
    }

    pub fn new_with_tmpdir<P: AsRef<path::Path>>(path: P, overwrite: OverwriteBehavior, tmpdir: P) -> Self {
        AtomicFile {
            path: path.as_ref().to_path_buf(),
            overwrite: overwrite,
            tmpdir: tmpdir.as_ref().to_path_buf()
        }
    }

    fn commit(&self, tmppath: &path::Path) -> io::Result<()> {
        match self.overwrite {
            AllowOverwrite => replace_atomic(tmppath, self.path()),
            DisallowOverwrite => move_atomic(tmppath, self.path())
        }
    }

    /// Get the target filepath.
    pub fn path(&self) -> &path::Path { &self.path.borrow() }


    /// Open a temporary file, call `f` on it (which is supposed to write to it), then move the
    /// file atomically to `self.path`.
    pub fn write<E, F: FnMut(&mut fs::File) -> io::Result<E>>(&self, mut f: F) -> io::Result<E> {
        let mut tmpfile = NamedTempFile::new_in(&self.tmpdir).ok().expect("Failed to create tmpfile");
        let rv = try!(f(&mut tmpfile));
        try!(tmpfile.sync_all());
        try!(self.commit(tmpfile.path()));
        mem::forget(tmpfile);  // Ensure file isn't cleaned up
        Ok(rv)
    }
}


#[cfg(unix)]
mod imp {
    use std::{io,fs,path};

    pub fn replace_atomic(src: &path::Path, dst: &path::Path) -> io::Result<()> {
        fs::rename(src, dst)
    }

    pub fn move_atomic(src: &path::Path, dst: &path::Path) -> io::Result<()> {
        try!(fs::hard_link(src, dst));
        fs::remove_file(src)
    }
}

#[cfg(windows)]
mod imp {
    extern crate winapi;
    extern crate kernel32 as win32kernel;

    use std::{io,path};
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    macro_rules! call {
        ($e: expr) => (
            if $e != 0 {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        )
    }

    fn path_to_windows_str<T: AsRef<OsStr>>(x: T) -> Vec<winapi::WCHAR> {
        x.as_ref().encode_wide().chain(Some(0)).collect()
    }

    pub fn replace_atomic(src: &path::Path, dst: &path::Path) -> io::Result<()> {
        call!(unsafe {win32kernel::MoveFileExW(
            path_to_windows_str(src).as_ptr(), path_to_windows_str(dst).as_ptr(),
            winapi::MOVEFILE_WRITE_THROUGH | winapi::MOVEFILE_REPLACE_EXISTING
        )})
    }

    pub fn move_atomic(src: &path::Path, dst: &path::Path) -> io::Result<()> {
        call!(unsafe {win32kernel::MoveFileExW(
            path_to_windows_str(src).as_ptr(), path_to_windows_str(dst).as_ptr(),
            winapi::MOVEFILE_WRITE_THROUGH
        )})
    }
}


/// Move `src` to `dst`. If `dst` exists, it will be silently overwritten.
///
/// Both paths must reside on the same filesystem for the operation to be atomic.
pub fn replace_atomic(src: &path::Path, dst: &path::Path) -> io::Result<()> {
    imp::replace_atomic(src, dst)
}


/// Move `src` to `dst`. An error will be returned if `dst` exists.
///
/// Both paths must reside on the same filesystem for the operation to be atomic.
pub fn move_atomic(src: &path::Path, dst: &path::Path) -> io::Result<()> {
    imp::move_atomic(src, dst)
}
