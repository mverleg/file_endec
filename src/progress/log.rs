use ::std::time::Instant;

use crate::files::file_meta::FileInfo;
use crate::header::{CompressionAlg, KeyHashAlg, SymmetricEncryptionAlg};
use crate::progress::Progress;

pub struct LogProgress {
    current: String,
    current_weight: usize,  //TODO @mark: TEMPORARY! REMOVE THIS!
    started_at: Instant,
}

impl LogProgress {
    pub fn new() -> Self {
        LogProgress {
            current: "initializing".to_owned(),
            current_weight: 1,
            started_at: Instant::now(),
        }
    }

    fn next_step(&mut self, next: String) {
        let duration = self.started_at.elapsed().as_millis();
        println!("> finish {} ({} ms)", &self.current, duration);
        println!("== {}: {} ms for {} units, so {:.3} ms/u", &self.current, duration, self.current_weight, (self.started_at.elapsed().as_micros() as f64 * 1e-3) / (self.current_weight as f64));
        println!("< start  {}", &next);
        self.current = next;
        self.started_at = Instant::now();
    }
}

impl Progress for LogProgress {
    fn start_stretch_alg(&mut self, alg: &KeyHashAlg, file: Option<&FileInfo>) {
        if let Some(file) = file {
            self.next_step(format!(
                "stretching key for {} using {}",
                file.file_name(),
                alg
            ));
        } else {
            self.next_step(format!("stretching key using {}", alg));
        }
    }

    fn start_read_for_file(&mut self, file: &FileInfo) {
        self.next_step(format!("reading {}", file.file_name()));
    }

    fn start_private_header_for_file(&mut self, file: &FileInfo) {
        self.next_step(format!("private header {}", file.file_name()));
    }

    fn start_compress_alg_for_file(&mut self, alg: &CompressionAlg, file: &FileInfo) {
        self.next_step(format!(
            "(de)compressing {} using {}",
            file.file_name(),
            alg
        ));
    }

    fn start_sym_alg_for_file(&mut self, alg: &SymmetricEncryptionAlg, file: &FileInfo) {
        self.next_step(format!("en/decrypting {} using {}", file.file_name(), alg));
    }

    fn start_checksum_for_file(&mut self, file: &FileInfo) {
        self.next_step(format!("calculating checksum {}", file.file_name()));
    }

    fn start_write_for_file(&mut self, file: &FileInfo) {
        self.next_step(format!(
            "writing {}",
            file.out_pth.file_name().unwrap().to_string_lossy()
        ));
    }

    fn start_shred_input_for_file(&mut self, file: &FileInfo) {
        self.next_step(format!("shred input {}", file.file_name()));
    }

    fn finish(&mut self) {
        self.next_step("finishing up".to_owned());
    }
}
