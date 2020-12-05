#![cfg(test)]

//TODO @mark: more e2e tests for all the v1.1 features

use ::std::env;
use ::std::fs;
use ::std::path::PathBuf;

use ::tempfile::{NamedTempFile, TempDir};

use crate::files::mockfile::write_test_file;
use crate::util::test_cmd::filename_append_enc;
use crate::util::test_cmd::test_decrypt;
use crate::util::test_cmd::test_encrypt;

/// This test is always enabled as a fast encrypy/decrypt cycle (using -s mode, a small file and -q).
#[test]
fn fast() {
    let key = "3Q#J3RwOIns@MK9TQDwZkpUK-EmH7T07";
    let (tmp, raw_pth, data) = write_test_file(1024);
    let enc_pth = filename_append_enc(raw_pth.as_path());
    test_encrypt(
        &[raw_pth.as_path()],
        &["-k", &format!("pass:{}", key), "-d", "-q", "-s", "--hide-meta"],
        None,
    );
    assert!(enc_pth.as_path().exists());
    assert!(!raw_pth.as_path().exists());
    test_decrypt(
        &[raw_pth.as_path()],
        &["-k", &format!("pass:{}", key), "-d", "-q"],
        None,
        true,
    );
    assert!(!enc_pth.as_path().exists());
    assert!(raw_pth.as_path().exists());
    assert_eq!(fs::read(raw_pth.as_path()).unwrap(), data);
    tmp.close().unwrap();
}

#[test]
#[cfg_attr(not(feature = "test-e2e"), ignore)]
fn large_file() {
    let key = "abc123";
    let (tmp, file, data) = write_test_file(128 * 1024 * 1024);
    let enc_pth = filename_append_enc(file.as_path());
    test_encrypt(
        &[file.as_path()],
        &[
            "-k",
            &format!("pass:{}", key),
            "--accept-weak-key",
            "-d",
            "-v",
        ],
        None,
    );
    assert!(enc_pth.as_path().exists());
    assert!(!file.as_path().exists());
    env::set_var("FED_E2E_LARGE_FILE_TESTKEY", key);
    test_decrypt(
        &[file.as_path()],
        &["-k", "env:FED_E2E_LARGE_FILE_TESTKEY", "-d", "-v"],
        None,
        true,
    );
    assert!(!enc_pth.as_path().exists());
    assert!(file.as_path().exists());
    assert_eq!(fs::read(file.as_path()).unwrap(), data);
    tmp.close().unwrap();
}

//TODO @mark: test that metadata is actually hidden (maybe not e2e?)

#[test]
#[cfg_attr(not(feature = "test-e2e"), ignore)]
fn mixed_options() {
    struct Options<'a> {
        size: usize,
        args: &'a [&'a str],
    }
    struct Encrypted {
        tmp: TempDir,
        path: PathBuf,
        data: Vec<u8>,
    }
    let key = "pass:G4yBazwH&iyuUK8qVjQwXIP%+s7VAS&j";
    let option_sets = [
        Options { size: 20 * 1024, args: &[] },
        Options { size: 100 * 1024, args: &["--fast",] },
        Options { size: 1024, args: &["--hide-meta", "--hide-size",] },
        Options { size: 128, args: &["-s", "--hide-meta"] },
    ];

    // Encrypt one by one.
    let mut encrypted = vec![];
    for options in &option_sets {
        let (tmp, file, data) = write_test_file(options.size);
        let enc_pth = filename_append_enc(file.as_path());
        let mut args = options.args.to_vec();
        args.push("-v");
        args.push("-k");
        args.push(key);
        test_encrypt(
            &[file.as_path()],
            &args,
            None,
        );
        assert!(enc_pth.as_path().exists());
        encrypted.push(Encrypted { tmp, path: enc_pth, data });
    }
    assert_eq!(option_sets.len(), encrypted.len());

    // Decrypt all at once, and verify.
    env::set_var("FED_E2E_OPTIONS_FILE_TESTKEY", key);
    let decrypt_paths = encrypted.iter().map(|enc| enc.path.as_path()).collect::<Vec<_>>();
    test_decrypt(
        &decrypt_paths,
        &["-k", "env:FED_E2E_OPTIONS_FILE_TESTKEY", "-v"],
        None,
        true,
    );
    encrypted.drain(|encrypted| {
        assert!(encrypt_info.path.as_path().exists());
        assert_eq!(fs::read(encrypt_info.path.as_path()).unwrap(), encrypt_info.data);
        (&encrypt_info.tmp).close().unwrap();
    });
}

#[test]
#[cfg_attr(not(feature = "test-e2e"), ignore)]
fn many_files() {
    let key = "!&R$ Eq1473L19XTGK'K7#be7Rl b62U8R2";
    let files: Vec<(TempDir, PathBuf, Vec<u8>)> =
        (10..60).map(|i| write_test_file(i * i)).collect();
    let paths: Vec<_> = files.iter().map(|f| f.1.as_path()).collect();
    test_encrypt(&paths, &["-k", "pipe", "-q"], Some(key.to_owned()));
    paths.iter().for_each(|p| assert!(p.exists()));
    test_decrypt(
        &paths,
        &["-k", "pipe", "-q", "-f"],
        Some(key.to_owned()),
        true,
    );
    for (_, path, data) in &files {
        assert!(filename_append_enc(&path).exists());
        assert!(path.exists());
        assert_eq!(&fs::read(&path).unwrap(), data);
    }
    files.into_iter().for_each(|f| f.0.close().unwrap());
}

#[test]
#[cfg_attr(not(feature = "test-e2e"), ignore)]
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
    test_encrypt(
        &[file.as_path()],
        &[
            "-k",
            &format!("file:{}", &key_pth.to_str().unwrap()),
            "--dry-run",
            "-d",
            "-f",
        ],
        None,
    );
    assert!(file.as_path().exists());
    assert!(collision_file.as_path().exists());
    assert_eq!(fs::read(&collision_file).unwrap(), b"hello world");
    dir.close().unwrap();
}

#[test]
#[cfg_attr(not(feature = "test-e2e"), ignore)]
fn output_dir_multi_salt() {
    let key = "!zEtt8M$vC6hJ9T@";
    let out_dir = TempDir::new().unwrap();
    let mut paths = vec![];
    for i in 0..5 {
        let extension = format!("e{0}{0}", i);
        let (dir, in_file, data) = write_test_file((10 + i) * 800);
        let mut out_file = out_dir.path().to_path_buf();
        out_file.push(format!(
            "{0}.{1}",
            in_file.file_name().unwrap().to_string_lossy(),
            &extension
        ));
        test_encrypt(
            &[in_file.as_path()],
            &[
                "-k",
                "pipe",
                "-v",
                "--output-dir",
                out_dir.path().to_str().unwrap(),
                "--output-extension",
                &extension,
            ],
            Some(key.to_owned()),
        );
        assert!(out_file.exists());
        paths.push((dir, in_file, out_file, data));
    }
    let out_paths: Vec<_> = paths.iter().map(|t| t.2.as_path()).collect();
    test_decrypt(
        &out_paths,
        &[
            "-k",
            &format!("pass:{}", key),
            "--output-dir",
            out_dir.path().to_str().unwrap(),
        ],
        None,
        false,
    );
    for path in paths {
        let enc_pth = {
            let mut p = path.2.clone();
            let name = p.file_name().unwrap().to_str().unwrap().to_owned();
            p.set_file_name(&name[..name.len() - 4].to_owned());
            p
        };
        assert!(enc_pth.exists());
        assert_eq!(fs::read(&enc_pth).unwrap(), path.3);
        path.0.close().unwrap();
    }
    out_dir.close().unwrap();
}

//TODO @mark: test wrong key (error msg) - should that be e2e?
