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

pub struct SilentProgress {}

impl SilentProgress {
    pub fn new() -> Self {
        SilentProgress {}
    }
}

impl Progress for SilentProgress {
    fn start_stretch_alg(&mut self, _alg: &KeyHashAlg, _file: Option<&FileInfo>) {}

    fn start_read_for_file(&mut self, _file: &FileInfo) {}

    fn start_compress_alg_for_file(&mut self, _alg: &CompressionAlg, _file: &FileInfo) {}

    fn start_sym_alg_for_file(&mut self, _alg: &SymmetricEncryptionAlg, _file: &FileInfo) {}

    fn start_checksum_for_file(&mut self, _file: &FileInfo) {}

    fn start_write_for_file(&mut self, _file: &FileInfo) {}

    fn finish(&mut self) {}
}
