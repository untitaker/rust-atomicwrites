# rust-atomicwrites

[![Build Status](https://travis-ci.org/untitaker/rust-atomicwrites.svg?branch=master)](https://travis-ci.org/untitaker/rust-atomicwrites)

Atomic file-writes to POSIX filesystems. The basic idea is to write to
temporary files, and move them when done writing. This avoids the problem of
two programs writing to the same file. For `AllowOverride`, `link + unlink` is
used instead of `rename` to raise errors when the target path already exists.

This is not in a state where you want to use it.

## Example

    use atomicwrites::{AtomicFile,DisallowOverwrite};

    let af = AtomicFile::new(&Path::new("foo"), DisallowOverwrite, None);
    try!(af.write(|&: f| {
        f.write_str("HELLO")
    }));

## License

Licensed under MIT, see ``LICENSE``.
