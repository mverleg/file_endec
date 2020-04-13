use std::collections::HashSet;

use ::indicatif::ProgressBar;

use crate::files::file_meta::FileInfo;
use crate::header::{Strategy, KeyHashAlg, SymmetricEncryptionAlg, CompressionAlg};
use crate::Verbosity;
use indicatif::ProgressStyle;

static RATIO_SYMMETRIC_STRETCH: f64 = 1.0;
static RATIO_COMPRESSION_STRETCH: f64 = 1.0;

#[derive(Debug, Hash, PartialEq)]
enum Task<'a> {
    Stretch(&'a KeyHashAlg),
    Compress(&'a SymmetricEncryptionAlg, FileInfo<'a>),
    Symmetric(&'a CompressionAlg, FileInfo<'a>),
}

#[derive(Debug)]
struct ProgressData {
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
        let bar = {
            let pb = ProgressBar::new(1000);
            pb.set_style(ProgressStyle::default_bar()
                .template("[{elapsed}] {wide_bar} [percent]")
                .progress_chars("##-"));
            pb.tick();
            pb
        };
        let mut todo = HashSet::new();
        for alg in &strategy.key_hash_algorithms {
            //strategy.stretch_count * RATIO_STRETCH_VS_KILOBYTE;
            todo.insert(Task::Stretch(alg.clone()));
        }
        for file in files {
            for alg in &strategy.symmetric_algorithms {
                todo.insert(Task::Symmetric(alg, file));
            }
        }
        for file in files {
            for alg in &strategy.compression_algorithm {
                todo.insert(Task::Compress(alg, file));
            }
        }
        Progress { data: Some(ProgressData {
            bar,
            current: None,
            todo,
        })}
    }

    pub fn start_stretch_alg(&mut self) {
        if let Some(data) = &self.data {

        }
    }

    pub fn start_compress_alg_for_file(&mut self) {
        if let Some(data) = &self.data {

        }
    }

    pub fn start_sym_alg_for_file(&mut self) {
        if let Some(data) = &self.data {

        }
    }

    pub fn finish(&mut self) {
        if let Some(data) = &self.data {
            debug_assert!(data.todo.is_empty());
            data.bar.finish();
        }
    }
}
