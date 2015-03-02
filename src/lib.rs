// DOCS

#![feature(old_path,old_io)]
use std::old_io;
pub use OverwriteBehavior::{AllowOverwrite, DisallowOverwrite};

pub trait GenericAtomicFile {
    /// Helper for writing to `path` in write-only mode.
    ///
    /// If `DisallowOverwrite` is given, errors will be returned from `self.write(...)` if the file
    /// exists.
    fn new(path: &Path, overwrite: OverwriteBehavior) -> Self;

    /// Get the target filepath.
    fn path(&self) -> &Path;

    /// Open a temporary file, call `f` on it (which is supposed to write to it), then move the
    /// file atomically to `self.path`.
    fn write<F: FnMut(&mut old_io::File) -> old_io::IoResult<()>>(&self, mut f: F) -> old_io::IoResult<()>;
}


#[derive(Copy)]
pub enum OverwriteBehavior {
    /// Overwrite files silently.
    AllowOverwrite,
    
    /// Don't overwrite files. `AtomicFile.write` will raise errors for such conditions only after
    /// you've already written your data.
    DisallowOverwrite
}

pub struct AtomicFile {
    path: Path,
    overwrite: OverwriteBehavior,
    tmpdir: Path
}


impl AtomicFile {
    pub fn new_with_tmpdir(path: &Path, overwrite: OverwriteBehavior, tmpdir: &Path) -> Self {
        AtomicFile {
            path: path.clone(),
            overwrite: overwrite,
            tmpdir: tmpdir.clone()
        }
    }

    fn commit(&self, tmppath: &Path) -> old_io::IoResult<()> {
        match self.overwrite {
            AllowOverwrite => replace_atomic(tmppath, self.path()),
            DisallowOverwrite => move_atomic(tmppath, self.path())
        }
    }
}


impl GenericAtomicFile for AtomicFile {
    fn new(path: &Path, overwrite: OverwriteBehavior) -> Self {
        AtomicFile::new_with_tmpdir(path, overwrite, &path.dir_path())
    }

    fn path(&self) -> &Path { &self.path }

    fn write<F: FnMut(&mut old_io::File) -> old_io::IoResult<()>>(&self, mut f: F) -> old_io::IoResult<()> {
        let tmpdir = try!(old_io::TempDir::new_in(&self.tmpdir, ".atomicwrite"));
        let tmppath = tmpdir.path().join(Path::new("tmpfile.tmp"));
        {
            let mut tmpfile = try!(old_io::File::create(&tmppath));
            try!(f(&mut tmpfile));
        };
        try!(self.commit(&tmppath));
        Ok(())
    }

}

#[cfg(unix)]
fn replace_atomic_impl(src: &Path, dst: &Path) -> old_io::IoResult<()> {
    old_io::fs::rename(src, dst)
}

#[cfg(unix)]
fn move_atomic_impl(src: &Path, dst: &Path) -> old_io::IoResult<()> {
    try!(old_io::fs::link(src, dst));
    old_io::fs::unlink(src)
}

/// Move `src` to `dst`. If `dst` exists, it will be silently overwritten.
///
/// Both paths must reside on the same filesystem for the operation to be atomic.
pub fn replace_atomic(src: &Path, dst: &Path) -> old_io::IoResult<()> {
    replace_atomic_impl(src, dst)
}

/// Move `src` to `dst`. An error will be returned if `dst` exists.
///
/// Both paths must reside on the same filesystem for the operation to be atomic.
pub fn move_atomic(src: &Path, dst: &Path) -> old_io::IoResult<()> {
    move_atomic_impl(src, dst)
}
