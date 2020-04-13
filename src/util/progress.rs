use std::collections::HashSet;

use ::indicatif::ProgressBar;
use ::indicatif::ProgressStyle;

use crate::files::file_meta::FileInfo;
use crate::header::{CompressionAlg, KeyHashAlg, Strategy, SymmetricEncryptionAlg};
use crate::Verbosity;

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
    size: u64,
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
        verbosity: &'a Verbosity,
        strategy: &'a Strategy,
        files: &'a [FileInfo],
    ) -> Self {
        if verbosity.quiet() {
            return Progress { data: None }
        }
        let mut todo = HashSet::new();
        //TODO @mark: consider adding load and save of file?
        for alg in &strategy.key_hash_algorithms {
            //strategy.stretch_count * RATIO_STRETCH_VS_KILOBYTE;
            todo.insert(Task {
                typ: TaskType::Stretch(alg),
                text: format!("{} key stretch", &alg),
                //TODO @mark: check the scaling here
                size: (strategy.stretch_count as f64 / 100.0).ceil() as u64,
            });
        }
        for file in files {
            for alg in &strategy.compression_algorithm {
                todo.insert(Task {
                    typ: TaskType::Compress(&alg, &file),
                    text: format!("{} {}", &alg, &file.file_name()),
                    //TODO @mark: check the scaling here
                    size: file.size_kb,
                });
            }
        }
        for file in files {
            for alg in &strategy.symmetric_algorithms {
                todo.insert(Task {
                    typ: TaskType::Symmetric(&alg, &file),
                    text: format!("{} {}", &alg, &file.file_name()),
                    //TODO @mark: check the scaling here
                    size: file.size_kb,
                });
            }
        }
        let total_size = todo.iter()
            .map(|task| task.size)
            .sum();
        let bar = {
            let pb = ProgressBar::new(total_size);
            pb.set_style(ProgressStyle::default_bar()
                .template("[{elapsed}] {wide_bar} [percent]")
                .progress_chars("##-"));
            pb.tick();
            pb
        };
        Progress { data: Some(ProgressData {
            bar,
            current: None,
            todo,
        })}
    }

    pub fn start_stretch_alg(&mut self, alg: &'a KeyHashAlg) {
        if let Some(data) = &self.data {

        }
    }

    pub fn start_compress_alg_for_file(&mut self, alg: &'a CompressionAlg, file: &'a FileInfo<'a>) {
        if let Some(data) = &self.data {

        }
    }

    pub fn start_sym_alg_for_file(&mut self, alg: &'a SymmetricEncryptionAlg, file: &'a FileInfo<'a>) {
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
