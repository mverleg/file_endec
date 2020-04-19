#![cfg(test)]

use crate::files::mockfile::write_test_file;
use crate::util::test_cmd::{test_encrypt, test_decrypt};

#[test]
fn large_file() {
    let file = write_test_file(128 * 1024 * 1024);
    let enc_out = test_encrypt(&vec![file.path()], &["-k", "pass:abc123qwerty987654321", "-d"]);
    let dec_out = test_decrypt(&vec![file.path()], &["-k", "pass:abc123qwerty987654321", "-d"]);

}

#[test]
fn many_files() {
    unimplemented!();
}

//TODO @mark: try all/most CLI args
//TODO @mark: can I try version compatibiltiy here?

