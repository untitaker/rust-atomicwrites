# rust-atomicwrites

Near-transactional file-writes to POSIX filesystems. This is not in a state
where you want to use this code without actually understanding it.

## Example

    use atomicwrites::{AtomicFile,DisallowOverwrite};

    let af = AtomicFile::new(&Path::new("foo"), DisallowOverwrite, None);
    try!(af.write(|&: f| {
        f.write_str("HELLO")
    }));
