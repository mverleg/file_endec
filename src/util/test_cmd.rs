#![cfg(any(test, feature = "expose"))]

use ::std::ffi::OsStr;
use ::std::io::Write;
use ::std::path::Path;
use ::std::path::PathBuf;
use ::std::process::Command;
use ::std::process::Stdio;
use ::std::str::from_utf8;

pub fn test_cmd<I, S>(args: I, input: Option<String>) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut ref_args = vec![];
    print!("cargo ");
    for arg in args.into_iter() {
        print!("{} ", arg.as_ref().to_string_lossy());
        ref_args.push(arg);
    }
    println!();
    let mut command = Command::new("cargo")
        .args(ref_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(txt) = input {
        command
            .stdin
            .as_mut()
            .unwrap()
            .write_all(txt.as_bytes())
            .unwrap();
        command.stdin.as_mut().unwrap().flush().unwrap();
    }
    let output = command.wait_with_output().unwrap();
    if !output.stderr.is_empty() {
        eprintln!("{}", from_utf8(&output.stderr).unwrap());
    }
    let out = from_utf8(&output.stdout).unwrap().to_owned();
    println!("{}", &out);
    assert!(output.status.success());
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

pub fn test_decrypt(
    paths: &[&Path],
    nonfile_args: &[&str],
    input: Option<String>,
    add_ext: bool,
) -> String {
    let mut args = vec![
        "run".to_owned(),
        "--release".to_owned(),
        "--bin".to_owned(),
        "filedec".to_owned(),
        "--".to_owned(),
    ];
    paths
        .iter()
        .map(|p| {
            if add_ext {
                filename_append_enc(p)
            } else {
                p.to_path_buf()
            }
        })
        .map(|p| p.to_str().unwrap().to_string())
        .for_each(|p| args.push(p));
    nonfile_args
        .into_iter()
        .for_each(|a| args.push((*a).to_owned()));
    test_cmd(args, input)
}

pub fn filename_append_enc(path: &Path) -> PathBuf {
    let mut p = path.to_owned();
    let name = path.file_name().unwrap().to_string_lossy();
    p.set_file_name(format!("{}.enc", name));
    p
}
