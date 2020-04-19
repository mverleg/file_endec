#![cfg(test)]

use ::std::process::Command;

use crate::files::mockfile::write_test_file;

#[test]
fn large_file() {
    let file = write_test_file(2 * 1024 * 1024 * 1024);
    let enc_out = Command::new("cargo")
        .args(&["run", "--release", "--bin", "fileenc", "--", file.path().to_str().unwrap(), "-k", "'pass:abc123qwerty987654321'", "-d"])
        .output()
        .unwrap();
    assert!(enc_out.status.success());
    let dec_out = Command::new("cargo")
        .args(&["run", "--release", "--bin", "filedec", "--", &format!("{}.enc", file.path().to_str().unwrap()), "-k", "'pass:abc123qwerty987654321'", "-d"])
        .output()
        .unwrap();
    assert!(dec_out.status.success());
}

#[test]
fn many_files() {
    unimplemented!();
}

//TODO @mark: try all/most CLI args
//TODO @mark: can I try version compatibiltiy here?

