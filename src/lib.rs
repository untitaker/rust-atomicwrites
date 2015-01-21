#![allow(unstable)]
use std::io;
pub use OverwriteBehavior::{AllowOverwrite, DisallowOverwrite};

pub struct AtomicFile {
    pub path: Path,
    pub overwrite: OverwriteBehavior,
    pub tmpdir: Path
}


impl AtomicFile {
    /// Open the given file in write-only mode, if not `allow_overwrite`, errors will be returned
    /// from `do(...)` if the file exists.
    ///
    /// The temporary directory defaults to `path`, and will be used for temporary files. It must
    /// reside on the same filesystem.
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
        self.overwrite {
            AllowOverwrite => io::fs::rename(tmppath, &self.path),
            DisallowOverwrite => io::fs::link(tmppath, &self.path)
        }
    }
}

#[derive(Copy)]
pub enum OverwriteBehavior {
    AllowOverwrite,
    DisallowOverwrite
}
