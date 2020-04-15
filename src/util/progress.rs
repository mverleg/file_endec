use ::std::collections::HashMap;

use ::indicatif::ProgressBar;
use ::indicatif::ProgressStyle;

use crate::files::file_meta::FileInfo;
use crate::header::{CompressionAlg, KeyHashAlg, Strategy, SymmetricEncryptionAlg};
use crate::Verbosity;
use std::path::PathBuf;
use std::mem;

//TODO @mark: TEMPORARY! REMOVE THIS!
static RATIO_SYMMETRIC_STRETCH: f64 = 1.0;
static RATIO_COMPRESSION_STRETCH: f64 = 1.0;

#[derive(Debug, Hash, PartialEq, Eq)]
enum TaskType {
    Stretch(KeyHashAlg),
    Compress(CompressionAlg, PathBuf),
    Symmetric(SymmetricEncryptionAlg, PathBuf),
}

#[derive(Debug)]
struct TaskInfo {
    text: String,
    size: u64,
}

#[derive(Debug)]
struct ProgressData {
    bar: ProgressBar,
    current: TaskInfo,
    todo: HashMap<TaskType, TaskInfo>,
}

impl  ProgressData {
    fn next_step(&mut self, next_task: Option<TaskInfo>) {
        let mut task = next_task.expect("attempted to start progress on a task that is not known");
        // Set the message of the task that is starting.
        self.bar.set_message(&task.text);
        // Swap the completed task and the starting one, so that a new task becomes
        // current, and 'task' is now the previous task.
        mem::swap(&mut task, &mut self.current);
        // Increment the progress bar based on the task that was just completed.
        self.bar.inc(task.size);
    }
}

pub trait Progress {
    fn start_stretch_alg(&mut self, alg: &KeyHashAlg);

    fn start_compress_alg_for_file(&mut self, alg: &CompressionAlg, file: &FileInfo);

    fn start_sym_alg_for_file(&mut self, alg: &SymmetricEncryptionAlg, file: &FileInfo);

    fn finish(&mut self);
}

#[derive(Debug)]
pub struct IndicatifProgress {
    data: Option<ProgressData>
}

impl  IndicatifProgress {
    pub fn new(
        verbosity: &Verbosity,
        strategy: &Strategy,
        files: &[FileInfo],
    ) -> Self {
        if verbosity.quiet() {
            return IndicatifProgress { data: None }
        }
        let mut todo = HashMap::new();
        //TODO @mark: consider adding load and save of file?
        for alg in &strategy.key_hash_algorithms {
            //strategy.stretch_count * RATIO_STRETCH_VS_KILOBYTE;
            todo.insert(
                TaskType::Stretch(alg.clone()),
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
                    TaskType::Compress(alg.clone(), file.in_path.to_owned()),
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
                    TaskType::Symmetric(alg.clone(), file.in_path.to_owned()),
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
                // .template("[{elapsed}] {msg:25<} [{wide_bar:}] {percent:>2}%")
                .template("[{wide_bar:}] {percent:>2}% {msg:25<}")
                .progress_chars("=> "));
            pb.tick();
            pb
        };
        IndicatifProgress { data: Some(ProgressData {
            bar,
            current: TaskInfo {
                text: "initialize".to_owned(),
                size: 1,
            },
            todo,
        })}
    }
}

impl Progress for IndicatifProgress {

    fn start_stretch_alg(&mut self, alg: &KeyHashAlg) {
        if let Some(ref mut data) = self.data {
            let typ = TaskType::Stretch(alg.clone());
            let info = data.todo.remove(&typ);
            data.next_step(info);
        }
    }

    fn start_compress_alg_for_file(&mut self, alg: &CompressionAlg, file: &FileInfo) {
        if let Some(data) = &mut self.data {
            let typ = TaskType::Compress(alg.clone(), file.in_path.to_owned());
            let info = data.todo.remove(&typ);
            data.next_step(info);
        }
    }

    fn start_sym_alg_for_file(&mut self, alg: &SymmetricEncryptionAlg, file: &FileInfo) {
        if let Some(data) = &mut self.data {
            let typ = TaskType::Symmetric(alg.clone(), file.in_path.to_owned());
            let info = data.todo.remove(&typ);
            data.next_step(info);
        }
    }

    fn finish(&mut self) {
        if let Some(data) = &mut self.data {
            debug_assert!(data.todo.is_empty());
            data.next_step(Some(TaskInfo {
                text: "finished".to_owned(),
                size: 0,
            }));
            data.bar.finish();
        }
    }
}

pub struct SilentProgress {}

impl SilentProgress {
    pub fn new() -> Self {
        SilentProgress {}
    }
}

impl Progress for SilentProgress {

    fn start_stretch_alg(&mut self, _alg: &KeyHashAlg) {}

    fn start_compress_alg_for_file(&mut self, _alg: &CompressionAlg, _file: &FileInfo) {}

    fn start_sym_alg_for_file(&mut self, _alg: &SymmetricEncryptionAlg, _file: &FileInfo) {}

    fn finish(&mut self) {}
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
        println!("finish {}", &self.current);
        println!("start  {}", &next);
        self.current = next;
    }
}

impl Progress for LogProgress {

    fn start_stretch_alg(&mut self, alg: &KeyHashAlg) {
        self.next(format!("stretching key using {}", alg));
    }

    fn start_compress_alg_for_file(&mut self, alg: &CompressionAlg, file: &FileInfo) {
        self.next(format!("(de)compressing {} using {}", file.file_name(), alg));
    }

    fn start_sym_alg_for_file(&mut self, alg: &SymmetricEncryptionAlg, file: &FileInfo) {
        self.next(format!("start en/decrypting {} using {}", file.file_name(), alg));
    }

    fn finish(&mut self) {
        self.next(format!("finishing up"));
    }
}
