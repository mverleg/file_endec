use crate::files::file_meta::FileInfo;
use crate::header::{CompressionAlg, KeyHashAlg, SymmetricEncryptionAlg};
use crate::progress::Progress;

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

    fn start_shred_input_for_file(&mut self, _file: &FileInfo) {}

    fn finish(&mut self) {}
}
