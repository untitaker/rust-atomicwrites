#![allow(unstable)]
use std::io;
use OverwriteBehavior::{AllowOverwrite, DisallowOverwrite};

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

    fn get_tmpfile(&self) -> io::IoResult<io::File> {
        let tmpdir = try!(io::TempDir::new_in(&self.tmpdir, "atomicwrite"));
        io::File::create(&tmpdir.path().join(Path::new("tmpfile.tmp")))
    }

    pub fn write(&self, f: &fn(&io::File) -> io::IoResult<()>) -> io::IoResult<()> {
        let mut tmpfile = try!(self.get_tmpfile());
        try!(f(&tmpfile));
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
