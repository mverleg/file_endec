#![cfg(test)]

use ::std::env;
use ::std::fs;
use ::std::path::PathBuf;

use ::tempfile::{NamedTempFile, TempDir};

use crate::files::mockfile::write_test_file;
use crate::util::test_cmd::{filename_append_enc, test_decrypt, test_encrypt};

#[test]
fn large_file() {
    let key = "abc123";
    let (tmp, file, data) = write_test_file(128 * 1024 * 1024);
    let enc_pth = filename_append_enc(file.as_path());
    test_encrypt(&vec![file.as_path()], &["-k", &format!("pass:{}", key), "--accept-weak-key", "-d", "-v"], None);
    assert!(enc_pth.as_path().exists());
    assert!(!file.as_path().exists());
    env::set_var("FED_E2E_LARGE_FILE_TESTKEY", key);
    test_decrypt(&vec![file.as_path()], &["-k", "env:FED_E2E_LARGE_FILE_TESTKEY", "-d", "-v"], None);
    assert!(!enc_pth.as_path().exists());
    assert!(file.as_path().exists());
    assert_eq!(fs::read(file.as_path()).unwrap(), data);
    tmp.close().unwrap();
}

#[test]
fn many_files() {
    let key = "!&R$ Eq1473L19XTGK'K7#be7Rl b62U8R2";
    let files: Vec<(TempDir, PathBuf, Vec<u8>)> = (10..60)
        .map(|i| write_test_file(i * i))
        .collect();
    let paths: Vec<_> = files.iter().map(|f| f.1.as_path()).collect();
    test_encrypt(&paths, &["-k", "pipe", "-q"], Some(key.to_owned()));
    paths.iter().for_each(|p| assert!(p.exists()));
    test_decrypt(&paths, &["-k", "pipe", "-q", "-f"], Some(key.to_owned()));
    for (_, path, data) in &files {
        assert!(filename_append_enc(&path).exists());
        assert!(path.exists());
        assert_eq!(&fs::read(&path).unwrap(), data);
    }
    files.into_iter().for_each(|f| f.0.close().unwrap());
}

#[test]
fn dry_run_passfile() {
    let key = "Lp0aY_=f9&zLEN-!D&jfdZPQH709-%N+";
    let (dir, file, _) = write_test_file(100 * 1024);
    // Key file
    let key_pth = NamedTempFile::new_in(dir.path()).unwrap().path().to_owned();
    fs::write(&key_pth, key.as_bytes()).unwrap();
    // File in output location
    let collision_file = filename_append_enc(&file);
    fs::write(&collision_file, b"hello world").unwrap();
    // Encrypt the test file
    test_encrypt(&vec![file.as_path()], &["-k", &format!("file:{}", &key_pth.to_str().unwrap()), "--dry-run", "-d", "-f"], None);
    assert!(file.as_path().exists());
    assert!(collision_file.as_path().exists());
    assert_eq!(fs::read(&collision_file).unwrap(), b"hello world");
    dir.close().unwrap();
}

#[test]
fn output_dir_multi_salt() {
    let key = "!zEtt8M$vC6hJ9T@";
    let out_dir = TempDir::new().unwrap();
    let mut paths = vec![];
    //TODO @mark: more iters
    for i in 0..1 {
        let extension = format!("e{0}{0}", i);
        let (dir, in_file, data) = write_test_file((10 + i) * 800);
        let mut out_file = in_file.clone();
        out_file.set_file_name(format!("{0}.{1}", in_file.file_name().unwrap().to_string_lossy(), &extension));
        test_encrypt(
            &vec![in_file.as_path()],
            &["-k", "pipe", "-v",
                "--output-dir", out_dir.path().to_str().unwrap(),
                "--output-extension", &extension],
            Some(key.to_owned())
        );
        paths.push((dir, in_file, out_file, data));
    }
    let out_paths: Vec<_> = paths.iter().map(|t| t.2.as_path()).collect();
    //TODO @mark: output-dir
    test_decrypt(&out_paths, &["-k", &format!("pass:{}", key)], None);

    paths.into_iter().for_each(|f| f.0.close().unwrap());
    out_dir.close().unwrap();

    // // Key file
    // let key_pth = NamedTempFile::new_in(out_dir.path()).unwrap().path().to_owned();
    // fs::write(&key_pth, key.as_bytes()).unwrap();
    // // File in output location
    // let collision_file = filename_append_enc(&file);
    // fs::write(&collision_file, b"hello world").unwrap();
    // // Encrypt the test file
    // test_encrypt(&vec![file.as_path()], &["-k", key], None);
    // test_decrypt(&vec![file1.as_path()], &["-k", key], None);
    // assert!(file.as_path().exists());
    // assert!(collision_file.as_path().exists());
    // assert_eq!(fs::read(&collision_file).unwrap(), b"hello world");
    // out_dir.close().unwrap();
}

//TODO @mark: test wrong key (error msg) - should that be e2e?
//TODO @mark: multiple files different keys
