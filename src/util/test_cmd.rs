#![cfg(any(test, feature = "expose"))]

use ::std::ffi::OsStr;
use ::std::path::Path;
use ::std::process::Command;
use ::std::str::from_utf8;

pub fn test_cmd<I, S>(args: I) -> String
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr> {
    let enc_out = Command::new("cargo")
        .args(args)
        .output()
        .unwrap();
    let err = from_utf8(&enc_out.stderr).unwrap();
    if !err.is_empty() {
        eprintln!("{}", err);
    }
    assert!(enc_out.status.success());
    let out = from_utf8(&enc_out.stdout).unwrap().to_owned();
    println!("{}", out);
    out
}

pub fn test_encrypt(paths: &[&Path], nonfile_args: &[&str]) -> String {
    let mut args = vec!["run", "--release", "--bin", "fileenc", "--"];
    for pth in paths {
        args.push(pth.to_str().unwrap());
    }
    args.extend_from_slice(nonfile_args);
    test_cmd(args)
}

pub fn test_decrypt(paths: &[&Path], nonfile_args: &[&str]) -> String {
    let mut args = vec![
        "run".to_owned(),
        "--release".to_owned(),
        "--bin".to_owned(),
        "filedec".to_owned(),
        "--".to_owned()];
    paths.iter()
        .map(|p| p.to_str().unwrap())
        .map(|p| format!("{}.enc", p))
        .for_each(|p| args.push(p));
    nonfile_args.into_iter()
        .for_each(|a| args.push((*a).to_owned()));
    test_cmd(args)
}
