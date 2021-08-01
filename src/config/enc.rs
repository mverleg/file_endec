use ::std::path::Path;
use ::std::path::PathBuf;

use crate::config::typ::{EndecConfig, InputAction, OnFileExist};
use crate::header::strategy::Verbosity;
use crate::key::Key;
use crate::util::option::EncOptionSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DryRun {
    IsReal,
    IsDryRun,
}

#[derive(Debug)]
pub struct EncryptConfig {
    files: Vec<PathBuf>,
    raw_key: Key,
    options: EncOptionSet,
    verbosity: Verbosity,
    overwrite: OnFileExist,
    delete_input: InputAction,
    output_dir: Option<PathBuf>,
    output_extension: String,
    dry_run: DryRun,
}

impl EncryptConfig {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        files: Vec<PathBuf>,
        raw_key: Key,
        options: EncOptionSet,
        verbosity: Verbosity,
        overwrite: OnFileExist,
        mut delete_input: InputAction,
        output_dir: Option<PathBuf>,
        output_extension: String,
        dry_run: DryRun,
    ) -> Self {
        assert!(!files.is_empty());
        if dry_run == DryRun::IsDryRun {
            delete_input = InputAction::Keep;
        }
        EncryptConfig {
            files,
            raw_key,
            options,
            verbosity,
            overwrite,
            delete_input,
            output_dir,
            output_extension,
            dry_run,
        }
    }

    pub fn options(&self) -> &EncOptionSet {
        &self.options
    }

    pub fn output_extension(&self) -> &str {
        &self.output_extension
    }

    pub fn dry_run(&self) -> bool {
        self.dry_run == DryRun::IsDryRun
    }
}

impl EndecConfig for EncryptConfig {
    fn files(&self) -> &[PathBuf] {
        &self.files
    }

    fn raw_key(&self) -> &Key {
        &self.raw_key
    }

    fn verbosity(&self) -> Verbosity {
        self.verbosity
    }

    fn overwrite(&self) -> bool {
        self.overwrite == OnFileExist::Overwrite
    }

    fn delete_input(&self) -> bool {
        self.delete_input == InputAction::Delete
    }

    fn output_dir(&self) -> Option<&Path> {
        match &self.output_dir {
            Some(dir) => Some(dir.as_path()),
            None => None,
        }
    }
}
