#![allow(unstable)]
use std::io;
use std::os;
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
        let mut tmpfile = try!(io::File::create(&tmppath));

        try!(f(&mut tmpfile));
        try!(tmpfile.fsync());
        try!(self.commit(tmpfile.path()));

        Ok(())
    }

    fn commit(&self, tmppath: &Path) -> io::IoResult<()> {
        match self.overwrite {
            AllowOverwrite => {
                try!(io::fs::rename(tmppath, &self.path));  // atomic
            },
            DisallowOverwrite => {
                try!(io::fs::link(tmppath, &self.path)); // atomic
                try!(io::fs::unlink(&self.path)); // doesn't matter if atomic
            }
        };
        Ok(())
    }
}

#[derive(Copy)]
pub enum OverwriteBehavior {
    AllowOverwrite,
    DisallowOverwrite
}
