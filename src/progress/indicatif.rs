use ::std::collections::HashMap;
use ::std::mem;
use ::std::path::PathBuf;

use ::indicatif::ProgressBar;
use ::indicatif::ProgressStyle;

use crate::files::file_meta::FileInfo;
use crate::files::read_headers::FileHeader;
use crate::files::read_headers::FileStrategy;
use crate::header::{CompressionAlg, KeyHashAlg, Strategy, SymmetricEncryptionAlg};
use crate::progress::Progress;
use crate::Verbosity;
use indicatif::ProgressDrawTarget;

#[derive(Debug, Hash, PartialEq, Eq)]
enum TaskType {
    Stretch(KeyHashAlg, Option<PathBuf>),
    Read(PathBuf),
    Compress(CompressionAlg, PathBuf),
    Symmetric(SymmetricEncryptionAlg, PathBuf),
    Checksum(PathBuf),
    Write(PathBuf),
    ShredInput(PathBuf),
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

impl<'a> FileStrategy for (&'a FileInfo<'a>, &'a Strategy) {
    fn file(&self) -> &FileInfo {
        self.0
    }

    fn strategy(&self) -> &Strategy {
        self.1
    }
}

#[derive(Debug)]
pub struct IndicatifProgress {
    data: Option<ProgressData>,
}

impl IndicatifProgress {
    fn new_file_strategy(
        is_enc: bool,
        file_strategies: &[impl FileStrategy],
        delete_input: bool,
        verbosity: Verbosity,
    ) -> Self {
        if verbosity.quiet() {
            return IndicatifProgress { data: None };
        }
        let mut todo = HashMap::new();
        for file_strat in file_strategies {
            for alg in &file_strat.strategy().key_hash_algorithms {
                let typ = if is_enc {
                    TaskType::Stretch(alg.clone(), None)
                } else {
                    TaskType::Stretch(alg.clone(), Some(file_strat.file().in_path.to_owned()))
                };
                todo.insert(
                    typ,
                    TaskInfo {
                        text: format!("{} key stretch", &alg),
                        size: file_strat.strategy().stretch_count * 6,
                    },
                );
            }
            todo.insert(
                TaskType::Read(file_strat.file().in_path.to_owned()),
                TaskInfo {
                    text: format!("read {}", &file_strat.file().file_name()),
                    size: file_strat.file().size_kb,
                },
            );
            todo.insert(
                TaskType::Write(file_strat.file().in_path.to_owned()),
                TaskInfo {
                    text: format!("write {}", file_strat.file().out_pth.file_name().unwrap().to_string_lossy()),
                    size: file_strat.file().size_kb * 2,
                },
            );
            if delete_input {
                todo.insert(
                    TaskType::ShredInput(file_strat.file().in_path.to_owned()),
                    TaskInfo {
                        text: format!("shred input {}", &file_strat.file().file_name()),
                        size: file_strat.file().size_kb * 3,
                    },
                );
            }
            for alg in &file_strat.strategy().compression_algorithm {
                todo.insert(
                    TaskType::Compress(alg.clone(), file_strat.file().in_path.to_owned()),
                    TaskInfo {
                        text: format!("{} {}", &alg, &file_strat.file().file_name()),
                        size: file_strat.file().size_kb * 3,
                    },
                );
            }
            for alg in &file_strat.strategy().symmetric_algorithms {
                todo.insert(
                    TaskType::Symmetric(alg.clone(), file_strat.file().in_path.to_owned()),
                    TaskInfo {
                        text: format!("{} {}", &alg, &file_strat.file().file_name()),
                        size: file_strat.file().size_kb * 3,
                    },
                );
            }
            todo.insert(
                TaskType::Checksum(file_strat.file().in_path.to_owned()),
                TaskInfo {
                    text: format!("checksum {}", &file_strat.file().file_name()),
                    size: file_strat.file().size_kb,
                },
            );
        }
        let total_size = todo.iter().map(|task| task.1.size).sum();
        let progress_bar = {
            let pb = ProgressBar::with_draw_target(total_size, ProgressDrawTarget::stderr());
            pb.set_style(
                ProgressStyle::default_bar()
                    // .template("[{elapsed}] {msg:25<} [{wide_bar:}] {percent:>2}%")
                    .template("[{wide_bar:}] {percent:>3}% {msg:<40!}")
                    .progress_chars("=> "),
            );
            pb.tick();
            pb
        };
        IndicatifProgress {
            data: Some(ProgressData {
                bar: progress_bar,
                current: TaskInfo {
                    text: "initialize".to_owned(),
                    size: 1,
                },
                todo,
            }),
        }
    }

    pub fn new_dec_strategy(file_strategies: &[FileHeader], delete_input: bool, verbosity: Verbosity) -> Self {
        IndicatifProgress::new_file_strategy(false, file_strategies, delete_input, verbosity)
    }

    pub fn new_enc_strategy<'a>(
        strategy: &'a Strategy,
        files: &'a [FileInfo],
        delete_input: bool,
        verbosity: Verbosity,
    ) -> Self {
        let file_strategies: Vec<_> = files.iter().map(|file| (file, strategy)).collect();
        IndicatifProgress::new_file_strategy(true, &file_strategies, delete_input, verbosity)
    }
}

impl Progress for IndicatifProgress {
    fn start_stretch_alg(&mut self, alg: &KeyHashAlg, file: Option<&FileInfo>) {
        if let Some(ref mut data) = self.data {
            let typ = if let Some(file) = file {
                TaskType::Stretch(alg.clone(), Some(file.in_path.to_owned()))
            } else {
                TaskType::Stretch(alg.clone(), None)
            };
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

    fn start_shred_input_for_file(&mut self, file: &FileInfo) {
        if let Some(data) = &mut self.data {
            let typ = TaskType::ShredInput(file.in_path.to_owned());
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