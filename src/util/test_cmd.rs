#![cfg(any(test, feature = "expose"))]

use ::std::ffi::OsStr;
use ::std::path::Path;
use ::std::process::Command;
use ::std::str::from_utf8;
use ::std::process::Stdio;
use ::std::io::{Read, Write};

pub fn test_cmd<I, S>(args: I, input: Option<String>) -> String
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr> {
    let mut command = Command::new("cargo")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(txt) = input {
        command.stdin.unwrap().write(txt.as_bytes());
    }
    let mut buffer = Vec::new();
    command.stderr.unwrap().read_to_end(&mut buffer).unwrap();
    let err = from_utf8(&buffer).unwrap();
    if !err.is_empty() {
        eprintln!("{}", err);
    }
    command.stdout.unwrap().read_to_end(&mut buffer).unwrap();
    let out = from_utf8(&buffer).unwrap().to_owned();
    println!("{}", &out);
    assert!(command.wait().unwrap().success());
    out
}

pub fn test_encrypt(paths: &[&Path], nonfile_args: &[&str], input: Option<String>) -> String {
    let mut args = vec!["run", "--release", "--bin", "fileenc", "--"];
    for pth in paths {
        args.push(pth.to_str().unwrap());
    }
    args.extend_from_slice(nonfile_args);
    test_cmd(args, input)
}

pub fn test_decrypt(paths: &[&Path], nonfile_args: &[&str], input: Option<String>) -> String {
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
    test_cmd(args, input)
}
