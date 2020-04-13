use std::collections::HashSet;

use ::indicatif::ProgressBar;

use crate::files::file_meta::FileInfo;
use crate::header::{Strategy, KeyHashAlg, SymmetricEncryptionAlg, CompressionAlg};
use crate::Verbosity;
use indicatif::ProgressStyle;

//TODO @mark: TEMPORARY! REMOVE THIS!
static RATIO_SYMMETRIC_STRETCH: f64 = 1.0;
static RATIO_COMPRESSION_STRETCH: f64 = 1.0;

#[derive(Debug, Hash, PartialEq, Eq)]
enum TaskType<'a> {
    Stretch(&'a KeyHashAlg),
    Compress(&'a CompressionAlg, &'a FileInfo<'a>),
    Symmetric(&'a SymmetricEncryptionAlg, &'a FileInfo<'a>),
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct Task<'a> {
    typ: TaskType<'a>,
    text: String,
    size: u32,
}

#[derive(Debug)]
struct ProgressData<'a> {
    bar: ProgressBar,
    current: Option<Task<'a>>,
    todo: HashSet<Task<'a>>,
}

#[derive(Debug)]
pub struct Progress<'a> {
    data: Option<ProgressData<'a>>
}

impl <'a> Progress<'a> {
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
        //TODO @mark: consider adding load and save of file?
        for alg in &strategy.key_hash_algorithms {
            //strategy.stretch_count * RATIO_STRETCH_VS_KILOBYTE;
            todo.insert(Task {
                typ: TaskType::Stretch(alg),
                text: format!("{} key stretch", &alg),
                size: todo!(),
            });
        }
        for file in files {
            for alg in &strategy.compression_algorithm {
                todo.insert(Task {
                    typ: TaskType::Compress(&alg, &file),
                    text: format!("{} {}", &alg, &file.file_name()),
                    size: todo!(),
                });
            }
        }
        for file in files {
            for alg in &strategy.symmetric_algorithms {
                todo.insert(Task {
                    typ: TaskType::Symmetric(&alg, &file),
                    text: format!("{} {}", &alg, &file.file_name()),
                    size: todo!(),
                });
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
