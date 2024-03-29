use ::std::env;
use ::std::fs;
use ::std::io::{stdin, BufRead};
use ::std::path::Path;
use ::std::path::PathBuf;
use ::std::str::FromStr;

use crate::key::Key;
use crate::util::FedResult;

#[derive(Debug, PartialEq, Eq)]
pub enum KeySource {
    CliArg(Key),
    EnvVar(String),
    File(PathBuf),
    AskTwice,
    AskOnce,
    Pipe,
}

impl FromStr for KeySource {
    type Err = String;

    fn from_str(txt: &str) -> Result<Self, Self::Err> {
        if let Some(stripped) = txt.strip_prefix("pass:") {
            return Ok(KeySource::CliArg(Key::new(stripped)));
        }
        if let Some(stripped) = txt.strip_prefix("env:") {
            return Ok(KeySource::EnvVar(stripped.to_owned()));
        }
        if let Some(stripped) = txt.strip_prefix("file:") {
            return Ok(KeySource::File(PathBuf::from(stripped.to_owned())));
        }
        if txt == "ask-once" || txt == "askonce" {
            return Ok(KeySource::AskOnce);
        }
        if txt == "ask" {
            return Ok(KeySource::AskTwice);
        }
        if txt == "pipe" {
            return Ok(KeySource::Pipe);
        }
        let txt_snip = if txt.len() > 5 {
            format!("{}...", txt[..4].to_owned())
        } else {
            txt.to_owned()
        };
        Err(format!(
            "key string was not recognized; got '{}', should be one of \
            'pass:$password', 'env:$var_name', 'file:$path', 'ask', 'ask-once', 'pipe'",
            txt_snip
        ))
    }
}

fn key_from_env_var(env_var_name: &str) -> FedResult<Key> {
    match env::var(env_var_name) {
        Ok(env_var_value) => Ok(Key::new(env_var_value.trim())),
        Err(err) => match err {
            env::VarError::NotPresent => Err(format!(
                "could not find environment variable named '{}' (which is \
                            expected to contain the encryption key)",
                env_var_name
            )),
            env::VarError::NotUnicode(_) => Err(format!(
                "environment variable named '{}' did not contain valid data (it \
                            is expected to contain the encryption key, which must be unicode)",
                env_var_name
            )),
        },
    }
}

fn key_from_file(file_path: &Path) -> FedResult<Key> {
    match fs::read_to_string(file_path) {
        Ok(content) => Ok(Key::new(content.trim())),
        Err(io_err) => Err(format!(
            "failed to read encryption key from \
                        file '{}'; reason: {}",
            file_path.to_string_lossy(),
            io_err
        )),
    }
}

fn ask_key_from_prompt(message: &str) -> FedResult<Key> {
    println!("{}", message);
    match rpassword::read_password() {
        Ok(pw) => {
            let pw = pw.trim();
            if pw.is_empty() {
                return Err("password from interactive console was empty".to_string());
            }
            Ok(Key::new(pw))
        }
        Err(_) => Err("failed to get password from interactive console".to_string()),
    }
}

fn key_from_prompt(ask_twice: bool) -> FedResult<Key> {
    if ask_twice {
        let pw1 = ask_key_from_prompt("key: ")?;
        let pw2 = ask_key_from_prompt("repeat key: ")?;
        if pw1 != pw2 {
            return Err("passwords did not match".to_owned());
        }
        Ok(pw2)
    } else {
        ask_key_from_prompt("key: ")
    }
}

fn key_from_pipe() -> FedResult<Key> {
    let mut pw = String::new();
    match stdin().lock().read_line(&mut pw) {
        Ok(count) => {
            if count >= 1 {
                Ok(Key::new(pw.trim()))
            } else {
                Err("no key was piped into the program".to_owned())
            }
        }
        Err(_) => Err("failed to read data piped into the program".to_owned()),
    }
}

impl KeySource {
    /// Obtain the key, which might involve IO.
    pub fn obtain_key(&self) -> FedResult<Key> {
        match self {
            KeySource::CliArg(pw) => Ok(pw.to_owned()),
            KeySource::EnvVar(env_var_name) => key_from_env_var(&env_var_name),
            KeySource::File(file_path) => key_from_file(&file_path),
            KeySource::AskOnce => key_from_prompt(false),
            KeySource::AskTwice => key_from_prompt(true),
            KeySource::Pipe => key_from_pipe(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_pass() {
        assert_eq!(
            KeySource::from_str("pass:secret").unwrap(),
            KeySource::CliArg(Key::new("secret")),
        );
    }

    #[test]
    fn valid_env() {
        assert_eq!(
            KeySource::from_str("env:varname").unwrap(),
            KeySource::EnvVar("varname".to_owned()),
        );
    }

    #[test]
    fn valid_file() {
        assert_eq!(
            KeySource::from_str("file:/my/pass.txt").unwrap(),
            KeySource::File(PathBuf::from("/my/pass.txt")),
        );
    }
    #[test]
    fn valid_ask_twice() {
        assert_eq!(KeySource::from_str("ask").unwrap(), KeySource::AskTwice,);
    }
    #[test]
    fn valid_ask_once() {
        assert_eq!(KeySource::from_str("ask-once").unwrap(), KeySource::AskOnce,);
    }
    #[test]
    fn valid_pipe() {
        assert_eq!(KeySource::from_str("pipe").unwrap(), KeySource::Pipe,);
    }

    #[test]
    fn invalid_key() {
        assert!(KeySource::from_str("hi").is_err());
    }
}
