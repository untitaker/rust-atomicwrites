// DOCS

#![feature(path,io)]
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

pub mod posix {
    use super::{GenericAtomicFile,OverwriteBehavior,AllowOverwrite,DisallowOverwrite};
    use std::old_io;

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
                AllowOverwrite => old_io::fs::rename(&tmppath, &self.path),
                DisallowOverwrite => old_io::fs::link(&tmppath, &self.path)
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
}

pub mod windows {
    use super::OverwriteBehavior;

    /// Currently a stub. Windows support is not implemented yet.
    pub struct AtomicFile {
        path: Path,
        overwrite: OverwriteBehavior
    }
}

pub use self::posix::AtomicFile as PosixAtomicFile;
pub use self::windows::AtomicFile as WindowsAtomicFile;


#[cfg(unix)]
pub use self::posix::AtomicFile as AtomicFile;

#[cfg(windows)]
pub use self::windows::AtomicFile as AtomicFile;
