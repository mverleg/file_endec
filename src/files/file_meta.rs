use ::std::fs;
use ::std::hash;
use ::std::io;
#[cfg(any(target_os = "linux", target_os = "l4re"))]
use ::std::os::unix::fs::PermissionsExt;
use ::std::path::Path;
use ::std::path::PathBuf;
use ::std::time::{SystemTime, UNIX_EPOCH};

use crate::config::typ::Extension;
use crate::header::strategy::Verbosity;
use crate::util::FedResult;
use crate::util::pth::determine_output_path;
use std::fs::Metadata;

#[derive(Debug)]
pub struct FileInfo<'a> {
    pub in_path: &'a Path,
    pub size_b: u64,
    /// octal notation, like 0o754 "rwxrw.r.."
    pub permissions: Option<u32>,
    pub created_ns: Option<u128>,
    pub changed_ns: Option<u128>,
    pub accessed_ns: Option<u128>,
    pub out_pth: PathBuf,
}

impl<'a> FileInfo<'a> {
    pub fn file_name(&self) -> String {
        self.in_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }

    pub fn size_kb(&self) -> u64 {
        self.size_b / 1024
    }
}

// Only relies on in_path, which should be uniquely identifying
impl<'a> hash::Hash for FileInfo<'a> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.in_path.hash(state)
    }
}

// Only relies on in_path, which should be uniquely identifying
impl<'a> PartialEq for FileInfo<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.in_path == other.in_path
    }
}

impl<'a> Eq for FileInfo<'a> {}

impl<'a> FileInfo<'a> {
    pub fn path_str(&self) -> String {
        self.in_path.to_string_lossy().to_string()
    }
}

fn to_ns(sys_time: io::Result<SystemTime>) -> Option<u128> {
    match sys_time {
        Ok(st) => match st.duration_since(UNIX_EPOCH) {
            Ok(duration) => Some(duration.as_nanos()),
            Err(_) => None,
        },
        Err(_) => None,
    }
}

#[cfg(any(target_os = "linux", target_os = "l4re"))]
fn get_perms(meta: &Metadata) -> Option<u32> {
    Some(meta.permissions().mode())
}

//TODO: maybe support more operating systems
#[cfg(not(any(target_os = "linux", target_os = "l4re")))]
fn get_perms() -> Option<u32> {
    None
}

pub fn inspect_files<'a>(
    files: &'a [PathBuf],
    verbosity: Verbosity,
    overwrite: bool,
    extension: Extension,
    output_dir: Option<&Path>,
) -> FedResult<Vec<FileInfo<'a>>> {
    let mut not_found_cnt: u32 = 0;
    let mut output_exists_cnt: u32 = 0;
    let mut infos = Vec::with_capacity(files.len());
    for file in files {
        // Input file
        let meta = match fs::metadata(file) {
            Ok(meta) => meta,
            Err(err) => {
                if verbosity.debug() {
                    eprintln!(
                        "could not read file '{}'; reason: {}",
                        file.to_string_lossy(),
                        err
                    )
                } else {
                    eprintln!("could not read file '{}'", file.to_string_lossy())
                }
                not_found_cnt += 1;
                continue;
            }
        };
        if !meta.is_file() {
            eprintln!("path '{}' is not a file", file.to_string_lossy());
            not_found_cnt += 1;
            continue;
        }

        // Output file
        let output_file = determine_output_path(file.as_path(), extension, output_dir);
        if !overwrite && output_file.exists() {
            eprintln!(
                "output path '{}' already exists",
                output_file.to_string_lossy()
            );
            output_exists_cnt += 1;
        }

        infos.push(FileInfo {
            in_path: file.as_path(),
            size_b: meta.len(),
            permissions: get_perms(&meta),
            created_ns: to_ns(meta.created()),
            changed_ns: to_ns(meta.modified()),
            accessed_ns: to_ns(meta.accessed()),
            out_pth: output_file,
        });
    }
    if not_found_cnt > 0 {
        return Err(format!(
            "aborting because {} input file{} not found",
            not_found_cnt,
            if not_found_cnt > 1 { "s were" } else { " was" }
        ));
    } else if output_exists_cnt > 0 {
        return Err(format!(
            "aborting because {} output file{} already exist (use --overwrite to overwrite, \
            or --output-dir {}to control output location)",
            output_exists_cnt,
            if output_exists_cnt > 1 { "s" } else { "" },
            if let Extension::Add(_) = extension {
                "or --output-extension "
            } else {
                ""
            },
        ));
    }
    Ok(infos)
}

#[cfg(test)]
mod tests {
    use ::tempfile::NamedTempFile;
    use ::tempfile::TempDir;

    use crate::config::EncryptConfig;
    use crate::config::typ::EndecConfig;
    use crate::config::typ::Extension;
    use crate::header::strategy::Verbosity;
    use crate::key::Key;
    use crate::util::option::EncOptionSet;

    use super::*;

    #[test]
    fn output_path() {
        let pth = TempDir::new().unwrap();
        let in_file_1 = NamedTempFile::new_in(pth.path()).unwrap();
        let in_file_2 = NamedTempFile::new_in(pth.path()).unwrap();
        let config = EncryptConfig::new(
            vec![in_file_1.path().to_owned(), in_file_2.path().to_owned()],
            Key::new("secret"),
            EncOptionSet::empty(),
            Verbosity::Debug,
            true,
            true,
            None,
            ".enc".to_owned(),
            false,
        );
        let out_files = inspect_files(
            config.files(),
            config.verbosity(),
            config.overwrite(),
            Extension::Add(".enc"),
            config.output_dir(),
        )
        .unwrap();
        assert_eq!(2, out_files.len());
        let expected_out_pth_1 = format!("{}.enc", in_file_1.path().to_str().unwrap());
        let expected_out_pth_2 = format!("{}.enc", in_file_2.path().to_str().unwrap());
        assert_eq!(out_files[0].out_pth.to_string_lossy(), expected_out_pth_1);
        assert_eq!(out_files[1].out_pth.to_string_lossy(), expected_out_pth_2);
    }
}
