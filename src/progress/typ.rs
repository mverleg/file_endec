use crate::files::file_meta::FileInfo;
use crate::header::{CompressionAlg, KeyHashAlg, SymmetricEncryptionAlg};

pub trait Progress {
    /// For encryption, stretching happens once, while for decryption, stretching pessimistically
    /// happens per file. As such, provide `file` for decryption, but not for encryption.
    fn start_stretch_alg(&mut self, alg: &KeyHashAlg, file: Option<&FileInfo>);

    fn start_read_for_file(&mut self, file: &FileInfo);

    fn start_compress_alg_for_file(&mut self, alg: &CompressionAlg, file: &FileInfo);

    fn start_sym_alg_for_file(&mut self, alg: &SymmetricEncryptionAlg, file: &FileInfo);

    fn start_checksum_for_file(&mut self, file: &FileInfo);

    fn start_write_for_file(&mut self, file: &FileInfo);

    fn start_shred_input_for_file(&mut self, file: &FileInfo);

    fn finish(&mut self);
}
