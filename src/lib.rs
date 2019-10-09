// INSERT_README_VIA_MAKE
extern crate tempdir;

use std::error::Error as ErrorTrait;
use std::fmt;
use std::io;
use std::fs;
use std::borrow::Borrow;
use std::path;
use std::convert::AsRef;

use tempdir::TempDir;

pub use OverwriteBehavior::{AllowOverwrite, DisallowOverwrite};


/// Whether to allow overwriting if the target file exists.
#[derive(Clone,Copy)]
pub enum OverwriteBehavior {
    /// Overwrite files silently.
    AllowOverwrite,

    /// Don't overwrite files. `AtomicFile.write` will raise errors for such conditions only after
    /// you've already written your data.
    DisallowOverwrite
}

/// Represents an error raised by `AtomicFile.write`.
#[derive(Debug)]
pub enum Error<E> {
    /// The error originated in the library itself, while it was either creating a temporary file
    /// or moving the file into place.
    Internal(io::Error),
    /// The error originated in the user-supplied callback.
    User(E)
}

/// If your callback returns a `std::io::Error`, you can unwrap this type to `std::io::Error`.
impl From<Error<io::Error>> for io::Error {
    fn from(e: Error<io::Error>) -> Self {
        match e {
            Error::Internal(x) => x,
            Error::User(x) => x
        }
    }
}

impl<E: fmt::Display> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Internal(ref e) => e.fmt(f),
            Error::User(ref e) => e.fmt(f)
        }
    }
}

impl<E: ErrorTrait> ErrorTrait for Error<E> {
    fn description(&self) -> &str {
        match *self {
            Error::Internal(ref e) => e.description(),
            Error::User(ref e) => e.description()
        }
    }

    fn cause(&self) -> Option<&ErrorTrait> {
        match *self {
            Error::Internal(ref e) => Some(e),
            Error::User(ref e) => Some(e)
        }
    }
}

fn safe_parent(p: &path::Path) -> Option<&path::Path> {
    match p.parent() {
        None => None,
        Some(x) if x.as_os_str().len() == 0 => Some(&path::Path::new(".")),
        x => x
    }
}

/// Create a file and write to it atomically, in a callback.
pub struct AtomicFile {
    /// Path to the final file that is atomically written.
    path: path::PathBuf,
    overwrite: OverwriteBehavior,
    /// Directory to which to write the temporary subdirectories.
    tmpdir: path::PathBuf
}


impl AtomicFile {
    /// Helper for writing to the file at `path` atomically, in write-only mode.
    ///
    /// If `OverwriteBehaviour::DisallowOverwrite` is given,
    /// an `Error::Internal` containing an `std::io::ErrorKind::AlreadyExists`
    /// will be returned from `self.write(...)` if the file exists.
    ///
    /// The temporary file is written to a temporary subdirectory in `.`, to ensure
    /// it’s on the same filesystem (so that the move is atomic).
    pub fn new<P>(path: P, overwrite: OverwriteBehavior) -> Self
    where P: AsRef<path::Path>
    {
        let p = path.as_ref();
        AtomicFile::new_with_tmpdir(p, overwrite, safe_parent(p).unwrap_or(path::Path::new(".")))
    }

    /// Like `AtomicFile::new`, but the temporary file is written to a temporary subdirectory in `tmpdir`.
    ///
    /// TODO: does `tmpdir` have to exist?
    pub fn new_with_tmpdir<P, Q>(path: P, overwrite: OverwriteBehavior, tmpdir: Q) -> Self
    where P: AsRef<path::Path>,
          Q: AsRef<path::Path>
    {
        AtomicFile {
            path: path.as_ref().to_path_buf(),
            overwrite: overwrite,
            tmpdir: tmpdir.as_ref().to_path_buf()
        }
    }

    /// Move the file to `self.path()`. Only call once! Not exposed!
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
    ///
    /// The temporary file is written to a randomized temporary subdirectory with prefix `.atomicwrite`.
    pub fn write<T, E, F>(&self, f: F) -> Result<T, Error<E>>
    where F: FnOnce(&mut fs::File) -> Result<T, E>
    {
        let tmpdir = TempDir::new_in(&self.tmpdir, ".atomicwrite").map_err(Error::Internal)?;

        let tmppath = tmpdir.path().join("tmpfile.tmp");
        let rv = {
            let mut tmpfile = fs::File::create(&tmppath).map_err(Error::Internal)?;
            let r = f(&mut tmpfile).map_err(Error::User)?;
            tmpfile.sync_all().map_err(Error::Internal)?;
            r
        };
        self.commit(&tmppath).map_err(Error::Internal)?;
        Ok(rv)
    }
}


#[cfg(unix)]
mod imp {
    extern crate nix;

    use super::safe_parent;

    use std::{io,fs,path};
    use std::os::unix::io::AsRawFd;

    fn fsync<T: AsRawFd>(f: T) -> io::Result<()> {
        match nix::unistd::fsync(f.as_raw_fd()) {
            Ok(()) => Ok(()),
            Err(nix::Error::Sys(errno)) => Err(errno.into()),
            Err(nix::Error::InvalidPath) => Err(io::Error::new(io::ErrorKind::Other, "invalid path")),
            Err(nix::Error::InvalidUtf8) => Err(io::Error::new(io::ErrorKind::Other, "invalid utf-8")),
            Err(nix::Error::UnsupportedOperation) => Err(io::Error::new(io::ErrorKind::Other, "unsupported operation"))
        }
    }

    fn fsync_dir(x: &path::Path) -> io::Result<()> {
        let f = fs::File::open(x)?;
        fsync(f)
    }

    pub fn replace_atomic(src: &path::Path, dst: &path::Path) -> io::Result<()> {
        fs::rename(src, dst)?;

        let dst_directory = safe_parent(dst).unwrap();
        fsync_dir(dst_directory)
    }

    pub fn move_atomic(src: &path::Path, dst: &path::Path) -> io::Result<()> {
        fs::hard_link(src, dst)?;
        fs::remove_file(src)?;

        let src_directory = safe_parent(src).unwrap();
        let dst_directory = safe_parent(dst).unwrap();
        fsync_dir(dst_directory)?;
        if src_directory != dst_directory { fsync_dir(src_directory)?; }
        Ok(())
    }
}

#[cfg(windows)]
mod imp {
    extern crate winapi;

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

    fn path_to_windows_str<T: AsRef<OsStr>>(x: T) -> Vec<winapi::shared::ntdef::WCHAR> {
        x.as_ref().encode_wide().chain(Some(0)).collect()
    }

    pub fn replace_atomic(src: &path::Path, dst: &path::Path) -> io::Result<()> {
        call!(unsafe {winapi::um::winbase::MoveFileExW(
            path_to_windows_str(src).as_ptr(), path_to_windows_str(dst).as_ptr(),
            winapi::um::winbase::MOVEFILE_WRITE_THROUGH | winapi::um::winbase::MOVEFILE_REPLACE_EXISTING
        )})
    }

    pub fn move_atomic(src: &path::Path, dst: &path::Path) -> io::Result<()> {
        call!(unsafe {winapi::um::winbase::MoveFileExW(
            path_to_windows_str(src).as_ptr(), path_to_windows_str(dst).as_ptr(),
            winapi::um::winbase::MOVEFILE_WRITE_THROUGH
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
