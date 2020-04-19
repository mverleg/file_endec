#![cfg(test)]

use ::std::path::Path;
use ::std::process::Command;
use ::std::str::from_utf8;

pub fn test_encrypt(paths: &[&Path], nonfile_args: &[&str]) -> String {
    let mut args = vec!["run", "--release", "--bin", "fileenc", "--"];
    for pth in paths {
        args.push(pth.to_str().unwrap());
    }
    args.extend_from_slice(nonfile_args);
    let enc_out = Command::new("cargo")
        .args(args)
        .output()
        .unwrap();
    assert!(enc_out.status.success());
    let err = from_utf8(&enc_out.stderr).unwrap();
    if !err.is_empty() {
        eprintln!("{}", err);
    }
    from_utf8(&enc_out.stdout).unwrap().to_owned()
}

pub fn test_decrypt(paths: &[&Path], nonfile_args: &[&str]) -> String {
    let mut args = vec![
        "run".to_owned(),
        "--release".to_owned(),
        "--bin".to_owned(),
        "fileenc".to_owned(),
        "--".to_owned()];
    paths.iter()
        .map(|p| p.to_str().unwrap())
        .map(|p| format!("{}.enc", p))
        .for_each(|p| args.push(p));
    nonfile_args.into_iter()
        .for_each(|a| args.push((*a).to_owned()));
    let enc_out = Command::new("cargo")
        .args(args)
        .output()
        .unwrap();
    assert!(enc_out.status.success());
    let err = from_utf8(&enc_out.stderr).unwrap();
    if !err.is_empty() {
        eprintln!("{}", err);
    }
    from_utf8(&enc_out.stdout).unwrap().to_owned()
}

