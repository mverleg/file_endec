use ::std::collections::HashMap;
use ::std::mem;
use ::std::path::PathBuf;
#[cfg(feature = "dev-mode")]
use ::std::time::Instant;

use ::indicatif::ProgressBar;
use ::indicatif::ProgressDrawTarget;
use ::indicatif::ProgressStyle;

use crate::files::file_meta::FileInfo;
use crate::files::read_headers::FileHeaderStrategy;
use crate::files::read_headers::FileStrategy;
use crate::header::{CompressionAlg, KeyHashAlg, Strategy, SymmetricEncryptionAlg};
use crate::progress::Progress;
use crate::Verbosity;

#[derive(Debug, Hash, PartialEq, Eq)]
enum TaskType {
    Stretch(KeyHashAlg, Option<PathBuf>),
    Read(PathBuf),
    PrivateHeader(PathBuf),
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
    #[cfg(feature = "dev-mode")]
    started_at: Instant,
}

impl ProgressData {
    fn next_step(&mut self, next_task: Option<TaskInfo>) {
        let mut task = next_task.expect("attempted to start progress on a task that is not known");

        #[cfg(feature = "dev-mode")]
        {
            let duration_us = self.started_at.elapsed().as_micros();
            let us_per_unit = (duration_us as f64 * 1e-3) / (self.current.size as f64);
            let msg = format!("{}: {} ms for {} units, so {:.4} ms/u", &self.current.text, duration_us/1000, self.current.size, us_per_unit);
            println!("== {}", &msg);
            eprintln!("== {}", &msg);
            self.started_at = Instant::now();
        }

        // Set the message of the task that is starting.
        self.bar.set_message(&task.text);
        // Increment the progress bar based on the task that was just completed.
        self.bar.inc(self.current.size);
        // Swap the completed task and the starting one, so that a new task becomes
        // current, and 'task' is now the previous task.
        mem::swap(&mut task, &mut self.current);
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
        // Note about sizes: I tuned them to be around 1 ms/unit to encrypt a 100MB file,
        // but of course that depends on hardware. As long as time per size unit is constant-ish.
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
                let weight = match alg {
                    KeyHashAlg::BCrypt => 50,
                    KeyHashAlg::Argon2i => 27,
                    KeyHashAlg::Sha512 => 33,
                };
                todo.insert(
                    typ,
                    TaskInfo {
                        text: format!("{} key stretch", &alg),
                        size: (file_strat.strategy().stretch_count + 1) * weight,
                    },
                );
            }
            todo.insert(
                TaskType::Read(file_strat.file().in_path.to_owned()),
                TaskInfo {
                    text: format!("read {}", &file_strat.file().file_name()),
                    size: file_strat.file().size_kb() / 2700 + 1,
                },
            );
            todo.insert(
                TaskType::Write(file_strat.file().in_path.to_owned()),
                TaskInfo {
                    text: format!(
                        "write {}",
                        file_strat
                            .file()
                            .out_pth
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                    ),
                    size: file_strat.file().size_kb() / 30 + 1,
                },
            );
            if delete_input {
                todo.insert(
                    TaskType::ShredInput(file_strat.file().in_path.to_owned()),
                    TaskInfo {
                        text: format!("shred input {}", &file_strat.file().file_name()),
                        size: file_strat.file().size_kb() / 29 + 1,
                    },
                );
            }
            todo.insert(
                TaskType::PrivateHeader(file_strat.file().in_path.to_owned()),
                TaskInfo {
                    text: format!("header {}", &file_strat.file().file_name()),
                    size: 1,
                },
            );
            for alg in &file_strat.strategy().compression_algorithm {
                let size_factor = match alg {
                    CompressionAlg::Brotli => 100,
                };
                todo.insert(
                    TaskType::Compress(alg.clone(), file_strat.file().in_path.to_owned()),
                    TaskInfo {
                        text: format!("{} {}", &alg, &file_strat.file().file_name()),
                        size: file_strat.file().size_kb() / size_factor + 1,
                    },
                );
            }
            for alg in &file_strat.strategy().symmetric_algorithms {
                let size_factor = match alg {
                    SymmetricEncryptionAlg::Aes256 => 35,
                    SymmetricEncryptionAlg::Twofish => 3,
                };
                todo.insert(
                    TaskType::Symmetric(alg.clone(), file_strat.file().in_path.to_owned()),
                    TaskInfo {
                        text: format!("{} {}", &alg, &file_strat.file().file_name()),
                        size: file_strat.file().size_kb() / size_factor + 1,
                    },
                );
            }
            todo.insert(
                TaskType::Checksum(file_strat.file().in_path.to_owned()),
                TaskInfo {
                    text: format!("checksum {}", &file_strat.file().file_name()),
                    size: file_strat.file().size_kb() / 170 + 1,
                },
            );
        }
        let total_size = todo.iter().map(|task| task.1.size).sum();
        let progress_bar = {
            let pb = ProgressBar::with_draw_target(total_size, ProgressDrawTarget::stderr());
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed:>3}] [{wide_bar:}] {percent:>3}% {msg:<40!}")
                    .progress_chars("=> "),
            );
            pb.tick();
            pb.enable_steady_tick(50);
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
                #[cfg(feature = "dev-mode")]
                started_at: Instant::now(),
            }),
        }
    }

    pub fn new_dec_strategy(
        file_strategies: &[FileHeaderStrategy],
        delete_input: bool,
        verbosity: Verbosity,
    ) -> Self {
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

    fn start_private_header_for_file(&mut self, file: &FileInfo) {
        if let Some(data) = &mut self.data {
            let typ = TaskType::PrivateHeader(file.in_path.to_owned());
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
