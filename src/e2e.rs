#![cfg(test)]

use crate::files::mockfile::write_test_file;
use crate::util::test_cmd::{test_encrypt, test_decrypt};
use tempfile::TempDir;
use std::path::PathBuf;

#[test]
fn large_file() {
    let (tmp, file) = write_test_file(128 * 1024 * 1024);
    test_encrypt(&vec![file.as_path()], &["-k", "pass:abc123qwerty987654321", "-d", "-v"]);
    test_decrypt(&vec![file.as_path()], &["-k", "pass:abc123qwerty987654321", "-d", "-v"]);
    tmp.close().unwrap()
}

#[test]
fn many_files() {
    let files: Vec<(TempDir, PathBuf)> = (50..200)
        .map(|i| write_test_file(i * 1024))
        .collect();
    let paths: Vec<_> = files.iter().map(|f| f.1.as_path()).collect();
    test_encrypt(&paths, &["-k", "pass:abc123qwerty987654321", "-v"]);
    test_decrypt(&paths, &["-k", "pass:abc123qwerty987654321", "-v"]);
    files.into_iter().for_each(|f| f.0.close().unwrap());
}

//TODO @mark: try all/most CLI args
//TODO @mark: can I try version compatibiltiy here?

