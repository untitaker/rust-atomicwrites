// DOCS

#![allow(unstable)]
use std::io;
pub use OverwriteBehavior::{AllowOverwrite, DisallowOverwrite};

pub struct AtomicFile {
    pub path: Path,
    pub overwrite: OverwriteBehavior,
    pub tmpdir: Path
}


impl AtomicFile {
    /// Helper for writing to `path` in write-only mode.
    ///
    /// If `DisallowOverwrite` is given, errors will be returned from `self.write(...)` if the file
    /// exists.  The `tmpdir` will be used for temporary files, and must reside on the same
    /// filesystem. It defaults to creating temporary files in the same directory as `path`.
    pub fn new(path: &Path, overwrite: OverwriteBehavior, tmpdir: Option<&Path>) -> AtomicFile {
        AtomicFile {
            path: path.clone(),
            overwrite: overwrite,
            tmpdir: match tmpdir {
                Some(x) => x.clone(),
                None => path.dir_path()
            }
        }
    }

    /// Open a temporary file, call `f` on it (which is supposed to write to it), then move the
    /// file atomically to `self.path`.
    pub fn write<F: FnMut(&mut io::File) -> io::IoResult<()>>(&self, mut f: F) -> io::IoResult<()> {
        let tmpdir = try!(io::TempDir::new_in(&self.tmpdir, ".atomicwrite"));
        let tmppath = tmpdir.path().join(Path::new("tmpfile.tmp"));
        {
            let mut tmpfile = try!(io::File::create(&tmppath));
            try!(f(&mut tmpfile));
        };
        try!(self.commit(&tmppath));

        Ok(())
    }

    /// Atomically move/copy the file to self.path.
    fn commit(&self, tmppath: &Path) -> io::IoResult<()> {
        match self.overwrite {
            AllowOverwrite => io::fs::rename(tmppath, &self.path),
            DisallowOverwrite => io::fs::link(tmppath, &self.path)
        }
    }
}

#[derive(Copy)]
pub enum OverwriteBehavior {
    /// Overwrite files silently.
    AllowOverwrite,
    
    /// Don't overwrite files. `AtomicFile.write` will raise errors for such conditions only after
    /// you've already written your data.
    DisallowOverwrite
}
