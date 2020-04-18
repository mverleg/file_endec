use ::std::collections::HashMap;
use ::std::mem;
use ::std::path::PathBuf;

use ::indicatif::ProgressBar;
use ::indicatif::ProgressStyle;

use crate::files::file_meta::FileInfo;
use crate::files::read_headers::FileHeader;
use crate::files::read_headers::FileStrategy;
use crate::header::{CompressionAlg, KeyHashAlg, Strategy, SymmetricEncryptionAlg};
use crate::Verbosity;
use crate::progress::Progress;

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
    fn start_stretch_alg(&mut self, alg: &KeyHashAlg, file: Option<&FileInfo>) {
        self.next(format!("stretching key for {} using {}", file.file_name(), alg));
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
