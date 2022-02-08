use ::std::fmt;
use ::std::io::stderr;
use ::std::io::Write;
use ::std::path::PathBuf;
use ::std::process::exit;

use ::structopt::StructOpt;

use ::file_endec::decrypt;
use ::file_endec::DecryptConfig;
use ::file_endec::FedResult;
use ::file_endec::Key;
use ::file_endec::KeySource;
use ::file_endec::Verbosity;

use ::std::time::SystemTime;

use ::derive_getters::Getters;
use ::env_logger;

use ::dockerfile_version_bumper::bump_dockerfiles;
use ::dockerfile_version_bumper::TagUp;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;


#[derive(Debug, StructOpt)]
#[structopt(
    name = "FileEnc",
    author = "github.com/mverleg/file_endec",
    about = "Securely encrypt one or more files using the given key."
)]
pub struct DecryptArguments {
    #[structopt(
        name = "FILES",
        parse(from_os_str),
        required = true,
        min_values = 1,
        help = "One or more paths to encrypted input files (absolute or relative)"
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
        help = "Delete encrypted input files after successful decryption."
    )]
    delete_input: bool,

    #[structopt(
        parse(from_os_str),
        short = "o",
        long,
        help = "Alternative output directory. If not given, output is saved alongside input."
    )]
    output_dir: Option<PathBuf>,
}

impl fmt::Display for DecryptArguments {
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
            if self.overwrite { "yes" } else { "no" }
        )?;

        writeln!(
            f,
            "* shred input: {}",
            if self.delete_input { "yes" } else { "no" }
        )?;

        Ok(())
    }
}

pub fn main() {
    let args = DecryptArguments::from_args();
    if let Err(err) = go_decrypt(args) {
        stderr().write_all(err.as_bytes()).unwrap();
        stderr().write_all(b"\n").unwrap();
        exit(1);
    }
}

impl DecryptArguments {
    fn convert(self, key: Key) -> FedResult<DecryptConfig> {
        let verbosity = match (self.debug, self.quiet) {
            (true, true) => return Err("cannot use quiet mode and debug mode together".to_owned()),
            (true, false) => Verbosity::Debug,
            (false, true) => Verbosity::Quiet,
            (false, false) => Verbosity::Normal,
        };
        Ok(DecryptConfig::new(
            self.files,
            key,
            verbosity,
            self.overwrite,
            self.delete_input,
            self.output_dir,
        ))
    }
}

//TODO: if wildcards or directories are ever supported, then skip files that have the encrypted extension (i.e. .enc)

fn go_decrypt(args: DecryptArguments) -> FedResult<()> {
    if args.debug {
        println!("arguments provided:\n{}", args);
    }
    let key = args.key_source.obtain_key()?;
    if args.debug {
        println!("approximate time to crack key: {}", key.time_to_crack());
    }
    let config = args.convert(key)?;
    decrypt(&config)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::file_endec::EndecConfig;

    #[test]
    fn parse_args_minimal() {
        let args = DecryptArguments::from_iter(&["fileenc", "file.txt"]);
        let config = args.convert(Key::new("abcdef123!")).unwrap();
        assert!(config.files().contains(&PathBuf::from("file.txt")));
        assert_eq!(config.raw_key().key_data.unsecure(), "abcdef123!");
        assert_eq!(config.verbosity(), Verbosity::Normal);
        assert_eq!(config.overwrite(), false);
        assert_eq!(config.delete_input(), false);
        assert_eq!(config.output_dir(), None);
    }

    #[test]
    fn parse_args_long() {
        let args = DecryptArguments::from_iter(&[
            "fileenc",
            "file.txt",
            "-q",
            "-d",
            "-f",
            "-o",
            "/tmp/hello",
        ]);
        let config = args.convert(Key::new("abcdef123!")).unwrap();
        assert!(config.files().contains(&PathBuf::from("file.txt")));
        assert_eq!(config.raw_key().key_data.unsecure(), "abcdef123!");
        assert_eq!(config.verbosity(), Verbosity::Quiet);
        assert_eq!(config.overwrite(), true);
        assert_eq!(config.delete_input(), true);
        assert_eq!(
            config.output_dir(),
            Some(PathBuf::from("/tmp/hello").as_path())
        );
    }
}
