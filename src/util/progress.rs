use ::std::collections::HashMap;

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

#[derive(Debug)]
struct TaskInfo {
    text: String,
    size: u64,
}

#[derive(Debug)]
struct ProgressData<'a> {
    bar: ProgressBar,
    current: Option<(TaskType<'a>, TaskInfo)>,
    todo: HashMap<TaskType<'a>, TaskInfo>,
}

pub trait Progress<'a> {
    fn start_stretch_alg(&mut self, alg: &'a KeyHashAlg);

    fn start_compress_alg_for_file(&mut self, alg: &'a CompressionAlg, file: &'a FileInfo<'a>);

    fn start_sym_alg_for_file(&mut self, alg: &'a SymmetricEncryptionAlg, file: &'a FileInfo<'a>);

    fn finish(&mut self);
}

#[derive(Debug)]
pub struct IndicatifProgress<'a> {
    data: Option<ProgressData<'a>>
}

impl <'a> IndicatifProgress<'a> {
    pub fn new(
        verbosity: &'a Verbosity,
        strategy: &'a Strategy,
        files: &'a [FileInfo],
    ) -> Self {
        if verbosity.quiet() {
            return IndicatifProgress { data: None }
        }
        let mut todo = HashMap::new();
        //TODO @mark: consider adding load and save of file?
        for alg in &strategy.key_hash_algorithms {
            //strategy.stretch_count * RATIO_STRETCH_VS_KILOBYTE;
            todo.insert(
                TaskType::Stretch(alg),
                TaskInfo {
                    text: format!("{} key stretch", &alg),
                    //TODO @mark: check the scaling here
                    size: (strategy.stretch_count as f64 / 100.0).ceil() as u64,
                }
            );
        }
        for file in files {
            for alg in &strategy.compression_algorithm {
                todo.insert(
                    TaskType::Compress(&alg, &file),
                    TaskInfo {
                        text: format!("{} {}", &alg, &file.file_name()),
                        //TODO @mark: check the scaling here
                        size: file.size_kb,
                    }
                );
            }
        }
        for file in files {
            for alg in &strategy.symmetric_algorithms {
                todo.insert(
                    TaskType::Symmetric(&alg, &file),
                    TaskInfo {
                        text: format!("{} {}", &alg, &file.file_name()),
                        //TODO @mark: check the scaling here
                        size: file.size_kb,
                    }
                );
            }
        }
        let total_size = todo.iter()
            .map(|task| task.1.size)
            .sum();
        let bar = {
            let pb = ProgressBar::new(total_size);
            pb.set_style(ProgressStyle::default_bar()
                .template("[{elapsed}] {wide_bar} [percent]")
                .progress_chars("##-"));
            pb.tick();
            pb
        };
        IndicatifProgress { data: Some(ProgressData {
            bar,
            current: None,
            todo,
        })}
    }
}

impl <'a> Progress<'a> for IndicatifProgress<'a> {

    fn start_stretch_alg(&mut self, alg: &'a KeyHashAlg) {
        if let Some(data) = &self.data {
            let typ = TaskType::Stretch(alg);
            let info = data.todo.get(&typ)
                .expect("attempted to start progress on a task that is not known");
            todo!();  //TODO @mark: TEMPORARY! REMOVE THIS!
        }
    }

    fn start_compress_alg_for_file(&mut self, alg: &'a CompressionAlg, file: &'a FileInfo<'a>) {
        if let Some(data) = &self.data {
            let typ = TaskType::Compress(&alg, &file);
            let info = data.todo.get(&typ)
                .expect("attempted to start progress on a task that is not known");
            todo!();  //TODO @mark: TEMPORARY! REMOVE THIS!
        }
    }

    fn start_sym_alg_for_file(&mut self, alg: &'a SymmetricEncryptionAlg, file: &'a FileInfo<'a>) {
        if let Some(data) = &self.data {
            let typ = TaskType::Symmetric(&alg, &file);
            let info = data.todo.get(&typ)
                .expect("attempted to start progress on a task that is not known");
            todo!();  //TODO @mark: TEMPORARY! REMOVE THIS!
        }
    }

    fn finish(&mut self) {
        if let Some(data) = &self.data {
            debug_assert!(data.todo.is_empty());
            data.bar.finish();
        }
    }
}

#[cfg(test)]
pub struct LogProgress {}

#[cfg(test)]
impl <'a> Progress<'a> for LogProgress {
    fn start_stretch_alg(&mut self, alg: &'a KeyHashAlg) {
        println!("stretching key using {}", alg);
    }

    fn start_compress_alg_for_file(&mut self, alg: &'a CompressionAlg, file: &'a FileInfo<'a>) {
        println!("(de)compressing {} using {}", file.file_name(), alg);
    }

    fn start_sym_alg_for_file(&mut self, alg: &'a SymmetricEncryptionAlg, file: &'a FileInfo<'a>) {
        println!("en/decrypting {} using {}", file.file_name(), alg);
    }

    fn finish(&mut self) {
        println!("finishing up");
    }
}
