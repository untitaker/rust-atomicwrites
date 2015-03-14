#![feature(core,io,path)]
extern crate atomicwrites;
extern crate tempdir;

use std::{fs,path};
use std::io::{Read,Write};
use atomicwrites::{AtomicFile,AllowOverwrite,DisallowOverwrite};
use tempdir::TempDir;

fn get_tmp() -> path::PathBuf {
    TempDir::new_in(".", "atomicwrites-test").unwrap().into_path()
}

#[test]
fn test_simple_allow_override() {
    let tmpdir = get_tmp();
    let path = tmpdir.join("haha");

    let af = AtomicFile::new(&path, AllowOverwrite);
    af.write(|f| f.write_all(b"HELLO")).unwrap();

    let mut rv = String::new();
    let mut testfd = fs::File::open(&path).unwrap();
    testfd.read_to_string(&mut rv).unwrap();
    assert_eq!(rv.as_slice(), "HELLO");
}

#[test]
fn test_simple_disallow_override() {
    let tmpdir = get_tmp();
    let path = tmpdir.join("haha");

    let af = AtomicFile::new(&path, DisallowOverwrite);
    af.write(|f| f.write_all(b"HELLO")).unwrap();

    let mut rv = String::new();
    let mut testfd = fs::File::open(&path).unwrap();
    testfd.read_to_string(&mut rv).unwrap();
    assert_eq!(rv.as_slice(), "HELLO");
}

#[test]
fn test_allowed_pathtypes() {
    AtomicFile::new("haha", DisallowOverwrite);
    AtomicFile::new(&"haha", DisallowOverwrite);
    AtomicFile::new(&path::Path::new("haha"), DisallowOverwrite);
    AtomicFile::new(&path::PathBuf::new("haha"), DisallowOverwrite);
}

#[test]
fn test_unicode() {
    let dmitri = "Дмитрий";
    let greeting = format!("HELLO {}", dmitri);

    let tmpdir = get_tmp();
    let path = tmpdir.join(dmitri);

    let af = AtomicFile::new(&path, DisallowOverwrite);
    af.write(|f| {
        f.write_all(greeting.as_bytes())
    }).unwrap();

    let mut rv = String::new();
    let mut testfd = fs::File::open(&path).unwrap();
    testfd.read_to_string(&mut rv).unwrap();
    assert_eq!(rv, greeting);
}
