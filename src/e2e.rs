#![cfg(test)]

use crate::files::mockfile::write_test_file;
use crate::util::test_cmd::{test_encrypt, test_decrypt, filename_append_enc};
use tempfile::{TempDir, NamedTempFile};
use std::path::PathBuf;
use std::fs;

#[ignore]  //TODO @mark: TEMPORARY! REMOVE THIS!
#[test]
fn large_file() {
    let key = "pass:abc123";
    let (tmp, file, data) = write_test_file(128 * 1024 * 1024);
    let enc_pth = filename_append_enc(file.as_path());
    test_encrypt(&vec![file.as_path()], &["-k", key, "--accept-weak-key", "-d", "-v"], None);
    assert!(enc_pth.as_path().exists());
    assert!(!file.as_path().exists());
    test_decrypt(&vec![file.as_path()], &["-k", key, "-d", "-v"], None);
    assert!(!enc_pth.as_path().exists());
    assert!(file.as_path().exists());
    assert_eq!(fs::read(file.as_path()).unwrap(), data);
    tmp.close().unwrap();
}

#[ignore]  //TODO @mark: TEMPORARY! REMOVE THIS!
#[test]
fn many_files() {
    let key = "!&R$ Eq1473L19XTGK'K7#be7Rl b62U8R2";
    let files: Vec<(TempDir, PathBuf, Vec<u8>)> = (10..60)
        .map(|i| write_test_file(i * i))
        .collect();
    let paths: Vec<_> = files.iter().map(|f| f.1.as_path()).collect();
    test_encrypt(&paths, &["-k", "pipe", "-q"], Some(key.to_owned()));
    paths.iter().for_each(|p| assert!(p.exists()));
    //TODO @mark: -q
    test_decrypt(&paths, &["-k", "pipe", "-v", "-f"], Some(key.to_owned()));
    for (_, path, data) in &files {
        assert!(filename_append_enc(&path).exists());
        assert!(path.exists());
        assert_eq!(&fs::read(&path).unwrap(), data);
    }
    files.into_iter().for_each(|f| f.0.close().unwrap());
}

//TODO @mark: many files different keys

#[ignore]  //TODO @mark: TEMPORARY! REMOVE THIS!
#[test]
fn dry_run_passfile() {
    let key = "pass:Lp0aY_=f9&zLEN-!D&jfdZPQH709-%N+";
    let (dir, file, _) = write_test_file(100 * 1024);
    // Key file
    let key_pth = NamedTempFile::new_in(dir.path()).unwrap().path().to_owned();
    fs::write(&key_pth, key.as_bytes()).unwrap();
    // File in output location
    let tmp = NamedTempFile::new_in(dir.path()).unwrap().path().to_owned();
    let collision_file = filename_append_enc(&tmp);
    fs::rename(&tmp, &collision_file).unwrap();
    fs::write(&collision_file, b"hello world").unwrap();
    // Encrypt the test file
    //TODO @mark: make sure dry_run does not overwrite file in target location
    test_encrypt(&vec![file.as_path()], &["-k", &format!("file:{}", &key_pth.to_str().unwrap()), "--dry-run", "-d", "-f"], None);
    //assert!(!enc_pth.as_path().exists());
    assert!(file.as_path().exists());
    dir.close().unwrap();
}

//TODO @mark: try all/most CLI args
//TODO @mark: can I try version compatibiltiy here?


//env:$var_name
//file:$path


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
//
// OPTIONS:
// -k, --key <key-source>           Where to get the key; one of 'pass:$password', 'env:$var_name', 'file:$path',
// 'ask', 'askonce', 'pipe' [default: ask]
// -o, --output-dir <output-dir>    Alternative output directory. If not given, output is saved alongside input.
//
// ARGS:
// <FILES>...    One or more paths to encrypted input files (absolute or relative)

