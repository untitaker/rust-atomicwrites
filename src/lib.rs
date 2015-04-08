// DOCS

extern crate tempdir;

use std::io;
use std::fs;
use std::borrow::Borrow;
use std::path;
use std::convert::AsRef;

use tempdir::TempDir;

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
        let tmpdir = match TempDir::new_in(
            &self.tmpdir,
            ".atomicwrite"
        ) {
            Ok(x) => x,
            Err(_) => return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to create a temporary directory."
            ))
        };

        let tmppath = tmpdir.path().join("tmpfile.tmp");
        let rv = try!({
            let mut tmpfile = try!(fs::File::create(&tmppath));
            f(&mut tmpfile)
        });
        try!(self.commit(&tmppath));
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
    extern crate "kernel32-sys" as win32kernel;

    use std::{io,os,path};
    use std::ffi::AsOsStr;
    use std::os::windows::OsStrExt;

    macro_rules! call {
        ($e: expr) => (
            if $e != 0 {
                Ok(())
            } else {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "A Windows API error occured.",
                    Some(os::last_os_error()),
                ))
            }
        )
    }

    fn path_to_windows_str(x: &path::Path) -> winapi::LPCWSTR {
        let v: Vec<winapi::WCHAR> = x.as_os_str().encode_wide().collect();
        v.as_slice().as_ptr()
    }

    pub fn replace_atomic(src: &path::Path, dst: &path::Path) -> io::Result<()> {
        call!(unsafe {win32kernel::MoveFileExW(
            path_to_windows_str(src), path_to_windows_str(dst),
            winapi::MOVEFILE_WRITE_THROUGH | winapi::MOVEFILE_REPLACE_EXISTING
        )})
    }

    pub fn move_atomic(src: &path::Path, dst: &path::Path) -> io::Result<()> {
        call!(unsafe {win32kernel::MoveFileExW(
            path_to_windows_str(src), path_to_windows_str(dst),
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
