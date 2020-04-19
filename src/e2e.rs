#![cfg(test)]

use crate::files::mockfile::write_test_file;
use crate::util::test_cmd::{test_encrypt, test_decrypt, append_enc};
use tempfile::TempDir;
use std::path::PathBuf;

#[test]
fn large_file() {
    let key = "pass:abc123";
    let (tmp, file) = write_test_file(128 * 1024 * 1024);
    let enc_pth = {
        let mut p = file.clone();
        p.set_file_name(format!("{}.enc", file.file_name().unwrap().to_string_lossy()));
        p
    };
    test_encrypt(&vec![file.as_path()], &["-k", key, "--accept-weak-key", "-d", "-v"], None);
    assert!(enc_pth.as_path().exists());
    assert!(!file.as_path().exists());
    test_decrypt(&vec![file.as_path()], &["-k", key, "-d", "-v"], None);
    assert!(!enc_pth.as_path().exists());
    assert!(file.as_path().exists());
    tmp.close().unwrap()
}

#[test]
fn many_files() {
    let key = "!&R$ Eq1\n473L19XTGK'K7#be7\0Rl b62U8R2";
    let files: Vec<(TempDir, PathBuf)> = (50..200)
        .map(|i| write_test_file(i * 1024))
        .collect();
    let paths: Vec<_> = files.iter().map(|f| f.1.as_path()).collect();
    test_encrypt(&paths, &["-k", "pipe", "-q"], Some(key.to_owned()));
    paths.iter().for_each(|p| assert!(p.exists()));
    test_decrypt(&paths, &["-k", "pipe", "-q"], Some(key.to_owned()));
    paths.iter().map(|p| append_enc(p)).for_each(|p| assert!(p.exists()));
    paths.iter().for_each(|p| assert!(p.exists()));
    files.into_iter().for_each(|f| f.0.close().unwrap());
}

//TODO @mark: try all/most CLI args
//TODO @mark: can I try version compatibiltiy here?


//pass:$password
//env:$var_name
//file:$path
//ask
//askonce
//pipe


// USAGE:
// fileenc [FLAGS] [OPTIONS] <FILES>...
//
// FLAGS:
// --accept-weak-key    Suppress warning if the encryption key is not strong.
// --dry-run            Test encryption, but do not save encrypted files (nor delete input, if --delete-input).
// -h, --help               Prints help information
// -f, --overwrite          Overwrite output files if they exist.
//
// OPTIONS:
// -k, --key <key-source>
// Where to get the key; one of 'pass:$password', 'env:$var_name', 'file:$path', 'ask', 'askonce', 'pipe'
// [default: ask]
// -o, --output-dir <output-dir>
// Alternative output directory. If not given, output is saved alongside input.
//
// --output-extension <output-extension>    Extension added to encrypted files. [default: .enc]
//
// ARGS:
// <FILES>...    One or more paths to input files (absolute or relative)
//
// @simba # :file_endec$ cargo run --release --bin filedec -- -h
// Finished release [optimized] target(s) in 0.13s
// Running `target/release/filedec -h`
// FileEnc 1.0.0
// github.com/mverleg/file_endec
// Securely encrypt one or more files using the given key.
//
// USAGE:
// filedec [FLAGS] [OPTIONS] <FILES>...
//
// FLAGS:
// -h, --help            Prints help information
// -f, --overwrite       Overwrite output files if they exist.
//
// OPTIONS:
// -k, --key <key-source>           Where to get the key; one of 'pass:$password', 'env:$var_name', 'file:$path',
// 'ask', 'askonce', 'pipe' [default: ask]
// -o, --output-dir <output-dir>    Alternative output directory. If not given, output is saved alongside input.
//
// ARGS:
// <FILES>...    One or more paths to encrypted input files (absolute or relative)

