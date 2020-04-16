use ::std::collections::HashMap;
use ::std::mem;
use ::std::path::PathBuf;

use ::indicatif::ProgressBar;
use ::indicatif::ProgressStyle;

use crate::files::file_meta::FileInfo;
use crate::header::{CompressionAlg, KeyHashAlg, Strategy, SymmetricEncryptionAlg};
use crate::Verbosity;

#[derive(Debug, Hash, PartialEq, Eq)]
enum TaskType {
    Stretch(KeyHashAlg),
    Read(PathBuf),
    Compress(CompressionAlg, PathBuf),
    Symmetric(SymmetricEncryptionAlg, PathBuf),
    Checksum(PathBuf),
    Write(PathBuf),
    DetailsPending(PathBuf)
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

//TODO @mark: conflicts warnig logging vs progress
impl ProgressData {
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

    fn start_read_for_file(&mut self, file: &FileInfo);

    fn start_compress_alg_for_file(&mut self, alg: &CompressionAlg, file: &FileInfo);

    fn start_sym_alg_for_file(&mut self, alg: &SymmetricEncryptionAlg, file: &FileInfo);

    fn start_checksum_for_file(&mut self, file: &FileInfo);

    fn start_write_for_file(&mut self, file: &FileInfo);

    fn finish(&mut self);
}

#[derive(Debug)]
pub struct IndicatifProgress {
    data: Option<ProgressData>,
}

impl IndicatifProgress {
    fn tasks_from_strategy(&mut self, file: &FileInfo) {
        todo.insert(
            TaskType::Read(file.in_path.to_owned()),
            TaskInfo {
                text: format!("read {}", &file.file_name()),
                size: file.size_kb,
            },
        );
        todo.insert(
            TaskType::Write(file.in_path.to_owned()),
            TaskInfo {
                text: format!("write {}", &file.file_name()),
                size: file.size_kb * 2,
            },
        );
        for alg in &strategy.compression_algorithm {
            todo.insert(
                TaskType::Compress(alg.clone(), file.in_path.to_owned()),
                TaskInfo {
                    text: format!("{} {}", &alg, &file.file_name()),
                    size: file.size_kb * 3,
                },
            );
        }
        for alg in &strategy.symmetric_algorithms {
            todo.insert(
                TaskType::Symmetric(alg.clone(), file.in_path.to_owned()),
                TaskInfo {
                    text: format!("{} {}", &alg, &file.file_name()),
                    size: file.size_kb * 3,
                },
            );
        }
        todo.insert(
            TaskType::Checksum(file.in_path.to_owned()),
            TaskInfo {
                text: format!("checksum {}", &file.file_name()),
                size: file.size_kb,
            },
        );
        //TODO @mark: make sure todos added here add up to the same total as the pending task
    }

    pub fn new(verbosity: &Verbosity, strategy: Option<&Strategy>, files: &[FileInfo]) -> Self {
        if verbosity.quiet() {
            return IndicatifProgress { data: None };
        }
        let mut todo = HashMap::new();
        for alg in &strategy.key_hash_algorithms {
            //strategy.stretch_count * RATIO_STRETCH_VS_KILOBYTE;
            todo.insert(
                TaskType::Stretch(alg.clone()),
                TaskInfo {
                    text: format!("{} key stretch", &alg),
                    //size: (strategy.stretch_count as f64 / 1.0).ceil() as u64,
                    size: strategy.stretch_count * 6,
                },
            );
        }
        for file in files {
            // Add 'pending' tasks, because for decryption, breakup per task isn't known in advance.

        }
        let total_size = todo.iter().map(|task| task.1.size).sum();
        let bar = {
            let pb = ProgressBar::new(total_size);
            pb.set_style(
                ProgressStyle::default_bar()
                    // .template("[{elapsed}] {msg:25<} [{wide_bar:}] {percent:>2}%")
                    .template("[{wide_bar:}] {percent:>3}% {msg:<40!}")
                    .progress_chars("=> "),
            );
            pb.tick();
            pb
        };
        let mut progress = IndicatifProgress {
            data: Some(ProgressData {
                bar,
                current: TaskInfo {
                    text: "initialize".to_owned(),
                    size: 1,
                },
                todo,
            }),
        };
        if let Some(strat) = strategy {
            // For encryption, the strategy is known beforehand, so break up
            // pending todos into detailed tasks immediately.
            for file in files {
                progress.tasks_from_strategy(&file);
            }
        }
        progress
    }
}

impl Progress for IndicatifProgress {
    fn start_stretch_alg(&mut self, alg: &KeyHashAlg) {
        if let Some(ref mut data) = self.data {
            let typ = TaskType::Stretch(alg.clone());
            match data.todo.remove(&typ) {
                // This is the first key stretch; use normal progress.
                Some(info) => data.next_step(Some(info)),
                // For decryption there could be an unpredictable amount of key stretching
                // because it is not known beforehand which salts are used. Use size 0.
                None => data.next_step(Some(TaskInfo {
                    text: format!("{} key stretch (again)", &alg),
                    size: 0,
                })),
            }
        }
    }

    fn start_read_for_file(&mut self, file: &FileInfo) {
        if let Some(data) = &mut self.data {
            let typ = TaskType::Read(file.in_path.to_owned());
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

    fn start_checksum_for_file(&mut self, file: &FileInfo) {
        if let Some(data) = &mut self.data {
            let typ = TaskType::Checksum(file.in_path.to_owned());
            let info = data.todo.remove(&typ);
            data.next_step(info);
        }
    }

    fn start_write_for_file(&mut self, file: &FileInfo) {
        if let Some(data) = &mut self.data {
            let typ = TaskType::Write(file.in_path.to_owned());
            let info = data.todo.remove(&typ);
            data.next_step(info);
        }
    }

    fn finish(&mut self) {
        if let Some(data) = &mut self.data {
            assert!(data.todo.is_empty());
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

    fn start_read_for_file(&mut self, _file: &FileInfo) {}

    fn start_compress_alg_for_file(&mut self, _alg: &CompressionAlg, _file: &FileInfo) {}

    fn start_sym_alg_for_file(&mut self, _alg: &SymmetricEncryptionAlg, _file: &FileInfo) {}

    fn start_checksum_for_file(&mut self, _file: &FileInfo) {}

    fn start_write_for_file(&mut self, _file: &FileInfo) {}

    fn finish(&mut self) {}
}

pub struct LogProgress {
    current: String,
}

impl LogProgress {
    pub fn new() -> Self {
        LogProgress {
            current: "initializing".to_owned(),
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

    fn start_read_for_file(&mut self, file: &FileInfo) {
        self.next(format!(
            "reading {}",
            file.file_name()
        ));
    }

    fn start_compress_alg_for_file(&mut self, alg: &CompressionAlg, file: &FileInfo) {
        self.next(format!(
            "(de)compressing {} using {}",
            file.file_name(),
            alg
        ));
    }

    fn start_sym_alg_for_file(&mut self, alg: &SymmetricEncryptionAlg, file: &FileInfo) {
        self.next(format!(
            "start en/decrypting {} using {}",
            file.file_name(),
            alg
        ));
    }

    fn start_checksum_for_file(&mut self, file: &FileInfo) {
        self.next(format!(
            "calculating checksum {}",
            file.file_name()
        ));
    }

    fn start_write_for_file(&mut self, file: &FileInfo) {
        self.next(format!(
            "writing {}",
            file.file_name()
        ));
    }

    fn finish(&mut self) {
        self.next(format!("finishing up"));
    }
}
