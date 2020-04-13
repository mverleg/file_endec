use std::collections::HashSet;

use ::indicatif::ProgressBar;

use crate::files::file_meta::FileInfo;
use crate::header::Strategy;
use crate::Verbosity;
use indicatif::ProgressStyle;

#[derive(Debug, Hash, PartialEq)]
struct Task {

}

#[derive(Debug)]
pub struct ProgressData {
    bar: ProgressBar,
    current: Option<Task>,
    todo: HashSet<Task>,
}

#[derive(Debug)]
pub struct Progress {
    data: Option<ProgressData>
}

impl Progress {
    pub fn new(
        verbosity: &Verbosity,
        strategy: &Strategy,
        files: &[FileInfo],
    ) -> Self {
        if verbosity.quiet() {
            return Progress { data: None }
        }
        //strategy.key_hash_algorithms
        //strategy.stretch_count
        //strategy.symmetric_algorithms
        //strategy.compression_algorithm
        let bar = {
            let pb = ProgressBar::new(1000);
            pb.set_style(ProgressStyle::default_bar()
                .template("[{elapsed}] {wide_bar} [percent]")
                .progress_chars("##-"));
            pb.tick();
            pb
        };
        let todo = ();
        Progress { data: Some(ProgressData {
            bar,
            current: None,
            todo,
        })}
    }

    pub fn start_stretch_alg(&mut self) {
        if let Some(data) = self.data {

        }
    }

    pub fn start_compress_alg_for_file(&mut self) {
        if let Some(data) = self.data {

        }
    }

    pub fn start_sym_alg_for_file(&mut self) {
        if let Some(data) = self.data {

        }
    }

    pub fn finish(&mut self) {
        if let Some(data) = self.data {
            debug_assert!(data.todo.is_empty());
            data.bar.finish();
        }
    }
}
