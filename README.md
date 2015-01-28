# rust-atomicwrites

[![Build Status](https://travis-ci.org/untitaker/rust-atomicwrites.svg?branch=master)](https://travis-ci.org/untitaker/rust-atomicwrites)

- [Documentation](http://rust-atomicwrites.unterwaditzer.net/)
- [Repository](https://github.com/untitaker/rust-atomicwrites)
- [Crates.io](https://crates.io/crates/atomicwrites)

Atomic file-writes to POSIX filesystems. The basic idea is to write to
temporary files, and move them when done writing. This avoids the problem of
two programs writing to the same file. For `AllowOverride`, `link + unlink` is
used instead of `rename` to raise errors when the target path already exists.

## Example

    use atomicwrites::{AtomicFile,DisallowOverwrite};

    let af = AtomicFile::new(&Path::new("foo"), DisallowOverwrite, None);
    try!(af.write(|&: f| {
        f.write_str("HELLO")
    }));


I'm not at all satisfied with this API, but there doesn't seem to be a
different way to force the user to check for errors when closing the file. [See
the relevant RFC
discussion](https://github.com/rust-lang/rfcs/pull/576/files#r23627669).

## License

Licensed under MIT, see ``LICENSE``.
