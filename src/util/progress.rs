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
    current: Option<TaskInfo>,
    todo: HashMap<TaskType<'a>, TaskInfo>,
}

impl <'a> ProgressData<'a> {
    fn next_step(&mut self, task: Option<TaskInfo>) {
        let task = task.expect("attempted to start progress on a task that is not known");
        let prev = &self.current.unwrap();
        self.bar.inc(prev.size);
        self.bar.set_message(&task.text);
        self.current = Some(task);
    }
}

pub trait Progress<'a> {
    fn start_stretch_alg(&mut self, alg: &KeyHashAlg);

    fn start_compress_alg_for_file(&mut self, alg: &CompressionAlg, file: &FileInfo<'a>);

    fn start_sym_alg_for_file(&mut self, alg: &SymmetricEncryptionAlg, file: &FileInfo<'a>);

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
                .template("[{elapsed}] {msg} {wide_bar} {percent}")
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

    fn start_stretch_alg(&mut self, alg: &KeyHashAlg) {
        if let Some(ref mut data) = self.data {
            let typ = TaskType::Stretch(alg);
            let info = data.todo.remove(&typ);
            data.next_step(info);
        }
    }

    fn start_compress_alg_for_file(&mut self, alg: &CompressionAlg, file: &FileInfo<'a>) {
        if let Some(data) = &mut self.data {
            let typ = TaskType::Compress(&alg, &file);
            let info = data.todo.remove(&typ);
            data.next_step(info);
        }
    }

    fn start_sym_alg_for_file(&mut self, alg: &SymmetricEncryptionAlg, file: &FileInfo<'a>) {
        if let Some(data) = &mut self.data {
            let typ = TaskType::Symmetric(&alg, &file);
            let info = data.todo.remove(&typ);
            data.next_step(info);
        }
    }

    fn finish(&mut self) {
        if let Some(data) = &self.data {
            debug_assert!(data.todo.is_empty());
            data.bar.finish();
        }
    }
}

pub struct LogProgress {
    current: String,
}

impl LogProgress {
    pub fn new() -> Self {
        LogProgress {
            current: "initializing".to_owned()
        }
    }

    fn next(&mut self, next: String) {
        println!("end {}", &self.current);
        println!("start {}", &next);
        self.current = next;
    }
}

impl <'a> Progress<'a> for LogProgress {

    fn start_stretch_alg(&mut self, alg: &KeyHashAlg) {
        self.next(format!("stretching key using {}", alg));
    }

    fn start_compress_alg_for_file(&mut self, alg: &CompressionAlg, file: &FileInfo<'a>) {
        self.next(format!("(de)compressing {} using {}", file.file_name(), alg));
    }

    fn start_sym_alg_for_file(&mut self, alg: &SymmetricEncryptionAlg, file: &FileInfo<'a>) {
        self.next(format!("start en/decrypting {} using {}", file.file_name(), alg));
    }

    fn finish(&mut self) {
        self.next(format!("finishing up"));
    }
}
