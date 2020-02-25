use ::std::fmt::Debug;
use ::std::path::Path;
use ::std::path::PathBuf;

use crate::header::strategy::Verbosity;
use crate::key::Key;

pub trait EndecConfig: Debug {
    fn files(&self) -> &[PathBuf];

    fn raw_key(&self) -> &Key;

    fn verbosity(&self) -> Verbosity;

    fn debug(&self) -> bool {
        Verbosity::Debug == self.verbosity()
    }

    fn quiet(&self) -> bool {
        Verbosity::Quiet == self.verbosity()
    }

    fn overwrite(&self) -> bool;

    fn delete_input(&self) -> bool;

    fn output_dir(&self) -> Option<&Path>;

    fn extension(&self) -> &str;
}
