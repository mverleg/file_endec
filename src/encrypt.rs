use ::std::fmt;
use ::std::path::PathBuf;
use ::std::process::exit;

use ::structopt::StructOpt;

use ::file_endec::EncOption;
use ::file_endec::encrypt;
use ::file_endec::EncryptConfig;
use ::file_endec::FedResult;
use ::file_endec::Key;
use ::file_endec::KeySource;
use ::file_endec::Verbosity;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "FileEnc",
    author = "github.com/mverleg/file_endec",
    about = "Securely encrypt one or more files using the given key."
)]
pub struct EncryptArguments {
    #[structopt(
        name = "FILES",
        parse(from_os_str),
        required = true,
        min_values = 1,
        help = "One or more paths to input files (absolute or relative)"
    )]
    files: Vec<PathBuf>,

    #[structopt(
        short = "k",
        long = "key",
        default_value = "ask",
        help = "Where to get the key; one of 'pass:$password', 'env:$var_name', 'file:$path', 'ask', 'ask-once', 'pipe'"
    )]
    key_source: KeySource,

    #[structopt(
        short = "v",
        long,
        help = "Show debug information, especially on errors."
    )]
    debug: bool,

    #[structopt(
        conflicts_with = "debug",
        short = "q",
        long = "quiet",
        help = "Do not show progress or other non-critical output."
    )]
    quiet: bool,

    #[structopt(short = "f", long, help = "Overwrite output files if they exist.")]
    overwrite: bool,

    #[structopt(
        short = "d",
        long,
        help = "Delete unencrypted input files after successful encryption (overwrites garbage before delete)."
    )]
    delete_input: bool,

    #[structopt(
        long,
        help = "Hide name, timestamp and permissions."
    )]
    hide_meta: bool,

    #[structopt(
        long,
        help = "Hide the exact compressed file size, by padding it to the next power of two."
    )]
    hide_size: bool,

    #[structopt(
        short = "s",
        long,
        help = "Use good instead of great encryption for a significant speedup."
    )]
    fast: bool,

    #[structopt(
        parse(from_os_str),
        short = "o",
        long,
        help = "Alternative output directory. If not given, output is saved alongside input."
    )]
    output_dir: Option<PathBuf>,

    #[structopt(
        long,
        default_value = ".enc",
        help = "Extension added to encrypted files."
    )]
    output_extension: String,

    #[structopt(
        long,
        help = "Test encryption, but do not save encrypted files (nor delete input, if --delete-input)."
    )]
    dry_run: bool,

    #[structopt(long, help = "Suppress warning if the encryption key is not strong.")]
    accept_weak_key: bool,
}

impl fmt::Display for EncryptArguments {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        writeln!(f, "* files:")?;
        for file in self.files.clone().into_iter() {
            writeln!(f, "  - {}", file.to_string_lossy().as_ref())?;
        }

        match &self.output_dir {
            Some(dir) => {
                writeln!(f, "* output: directory {}", dir.to_string_lossy().as_ref())?;
            }
            None => writeln!(
                f,
                "* output: stored alongside input (no directory requested)"
            )?,
        }

        writeln!(f, "* extension: {}", &self.output_extension)?;

        writeln!(f, "* hide metadata: {}", if self.hide_meta { "yes" } else { "no" })?;
        writeln!(f, "* hide size: {}", if self.hide_meta { "yes" } else { "no" })?;

        writeln!(f, "* fast mode: {}", if self.fast { "YES" } else { "no" })?;

        writeln!(
            f,
            "* logging: {}",
            if self.debug {
                "verbose"
            } else if self.quiet {
                "quiet"
            } else {
                "normal"
            }
        )?;

        writeln!(
            f,
            "* overwrite existing output: {}",
            if self.overwrite {
                if self.dry_run {
                    "no (overridden by dry run)"
                } else {
                    "yes"
                }
            } else {
                "no"
            }
        )?;

        writeln!(
            f,
            "* shred input: {}",
            if self.delete_input {
                if self.dry_run {
                    "no (overridden by dry run)"
                } else {
                    "yes"
                }
            } else {
                "no"
            }
        )?;

        writeln!(
            f,
            "* weak keys: {}",
            if self.accept_weak_key {
                "accept"
            } else {
                "warn"
            }
        )?;

        Ok(())
    }
}

pub fn main() {
    if let Err(err) = go_encrypt() {
        eprintln!("{}", err);
        exit(1);
    }
}

impl EncryptArguments {
    fn convert(self, key: Key) -> FedResult<EncryptConfig> {
        let verbosity = match (self.debug, self.quiet) {
            (true, true) => return Err("cannot use quiet mode and debug mode together".to_owned()),
            (true, false) => Verbosity::Debug,
            (false, true) => Verbosity::Quiet,
            (false, false) => Verbosity::Normal,
        };
        let mut options = vec![];
        if self.fast {
            options.push(EncOption::Fast);
        }
        if self.hide_meta {
            options.push(EncOption::HideMeta);
        }
        if self.hide_size {
            options.push(EncOption::PadSize);
        }
        let extension = if self.output_extension.starts_with('.') {
            self.output_extension
        } else {
            format!(".{}", self.output_extension)
        };
        Ok(EncryptConfig::new(
            self.files,
            key,
            options.into(),
            verbosity,
            self.overwrite,
            self.delete_input,
            self.output_dir,
            extension,
            self.dry_run,
        ))
    }
}

//TODO: if wildcards or directories are ever supported, then skip files that have the encrypted extension (i.e. .enc)

fn go_encrypt() -> FedResult<()> {
    let args = EncryptArguments::from_args();
    if args.debug {
        println!("arguments provided:\n{}", args);
    }
    let key = args.key_source.obtain_key()?;
    if args.debug {
        println!("approximate time to crack key: {}", key.time_to_crack());
    }
    if !args.accept_weak_key && !key.is_strong() {
        eprintln!(
            "warning: the encryption key is not strong (it might be cracked in {})",
            key.time_to_crack()
        );
    }
    let config = args.convert(key)?;
    encrypt(&config)
}

#[cfg(test)]
mod tests {
    use ::file_endec::EndecConfig;

    use super::*;

    #[test]
    fn parse_args_minimal() {
        let args = EncryptArguments::from_iter(&["fileenc", "file.txt"]);
        let config = args.convert(Key::new("abcdef123!")).unwrap();
        assert!(config.files().contains(&PathBuf::from("file.txt")));
        assert_eq!(1, config.files().len());
        assert_eq!(config.raw_key().key_data.unsecure(), "abcdef123!");
        assert_eq!(config.verbosity(), Verbosity::Normal);
        assert_eq!(config.overwrite(), false);
        assert_eq!(config.delete_input(), false);
        assert_eq!(config.output_dir(), None);
        assert_eq!(config.output_extension(), ".enc");
        assert_eq!(config.dry_run(), false);
    }

    #[test]
    fn parse_args_long() {
        let args = EncryptArguments::from_iter(&[
            "fileenc",
            "file.txt",
            "-q",
            "-d",
            "-f",
            "-s",
            "--hide-meta",
            "-o",
            "/tmp/hello",
            "--output-extension",
            "secret",
            "another_file.txt",
            "there_are_three_files",
        ]);
        let config = args.convert(Key::new("abcdef123!")).unwrap();
        assert!(config.files().contains(&PathBuf::from("file.txt")));
        assert!(config.files().contains(&PathBuf::from("another_file.txt")));
        assert!(config
            .files()
            .contains(&PathBuf::from("there_are_three_files")));
        assert_eq!(3, config.files().len());
        assert_eq!(config.raw_key().key_data.unsecure(), "abcdef123!");
        assert_eq!(config.verbosity(), Verbosity::Quiet);
        assert_eq!(config.overwrite(), true);
        assert_eq!(config.delete_input(), true);
        assert_eq!(
            config.output_dir(),
            Some(PathBuf::from("/tmp/hello").as_path())
        );
        assert_eq!(config.output_extension(), ".secret");
        assert_eq!(config.dry_run(), false);
    }
}
