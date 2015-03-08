#![feature(core,io,path)]
extern crate atomicwrites;
extern crate tempdir;

use std::{fs,path};
use std::io::{Read,Write};
use atomicwrites::{GenericAtomicFile,AtomicFile,AllowOverwrite,DisallowOverwrite};
use tempdir::TempDir;

fn get_tmp() -> path::PathBuf {
    TempDir::new("atomicwrites-test").unwrap().into_path()
}

#[test]
fn test_simple_allow_override() {
    let tmpdir = get_tmp();
    let path = tmpdir.join(&path::Path::new("haha"));

    let af: AtomicFile = GenericAtomicFile::new(&path, AllowOverwrite);
    af.write(|f| f.write_all(b"HELLO")).unwrap();

    let mut rv = String::new();
    let mut testfd = fs::File::open(&path).unwrap();
    testfd.read_to_string(&mut rv).unwrap();
    assert_eq!(rv.as_slice(), "HELLO");
}

#[test]
fn test_simple_disallow_override() {
    let tmpdir = get_tmp();
    let path = tmpdir.join(&path::Path::new("haha"));

    let af: AtomicFile = GenericAtomicFile::new(&path, DisallowOverwrite);
    af.write(|f| f.write_all(b"HELLO")).unwrap();

    let mut rv = String::new();
    let mut testfd = fs::File::open(&path).unwrap();
    testfd.read_to_string(&mut rv).unwrap();
    assert_eq!(rv.as_slice(), "HELLO");
}

