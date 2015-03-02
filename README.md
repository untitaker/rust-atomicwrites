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

    use atomicwrites::{AtomicFile,GenericAtomicFile,DisallowOverwrite};

    let af: AtomicFile = GenericAtomicFile::new(&Path::new("foo"), DisallowOverwrite);
    try!(af.write(|f| {
        f.write_str("HELLO")
    }));

Similar to `std::path`, `AtomicFile` should be used unless platform-specific
code is written. In order to use any of `AtomicFile`'s methods, you also have
to `use GenericAtomicFile` for now.

I'm not at all satisfied with this API, but there doesn't seem to be a
different way to force the user to check for errors when closing the file. [See
the relevant RFC discussion](https://github.com/rust-lang/rfcs/pull/576),
suggestions in the issue tracker on how to improve the API are welcome too.

## License

Licensed under MIT, see ``LICENSE``.
