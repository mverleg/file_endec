pub use crate::config::DecryptConfig;
pub use crate::config::EncryptConfig;
pub use crate::config::EndecConfig;
#[cfg(feature = "expose")]
pub use crate::files::mockfile::generate_test_file_content_for_test;
pub use crate::header::strategy::get_current_version_strategy;
pub use crate::header::strategy::Verbosity;
#[cfg(feature = "expose")]
pub use crate::key::hash::hash_argon2i;
#[cfg(feature = "expose")]
pub use crate::key::hash::hash_bcrypt;
#[cfg(feature = "expose")]
pub use crate::key::hash::hash_sha256;
#[cfg(feature = "expose")]
pub use crate::key::key::StretchKey;
#[cfg(feature = "expose")]
pub use crate::key::stretch::stretch_key;
pub use crate::key::Key;
pub use crate::key::KeySource;
#[cfg(feature = "expose")]
pub use crate::key::Salt;
pub use crate::orchestrate::decrypt::decrypt;
pub use crate::orchestrate::encrypt::encrypt;
#[cfg(feature = "expose")]
pub use crate::symmetric::decrypt::decrypt_aes256;
#[cfg(feature = "expose")]
pub use crate::symmetric::decrypt::decrypt_twofish;
#[cfg(feature = "expose")]
pub use crate::symmetric::encrypt::encrypt_aes256;
#[cfg(feature = "expose")]
pub use crate::symmetric::encrypt::encrypt_twofish;
pub use crate::util::option::EncOption;
pub use crate::util::option::EncOptionSet;
pub use crate::util::FedResult;

//TODO @mark: match set_file_times(path, FileTime::zero(), FileTime::zero()) {

mod config;
mod e2e;
mod files;
mod header;
mod key;
mod orchestrate;
mod progress;
mod symmetric;
mod util;

#[cfg(test)]
mod tests {
    use ::aes::Aes256;
    use ::secstr::SecVec;

    type Aes256Cbc = Cbc<Aes256, Iso7816>;

    /// The demo used in this blog post:
    /// https://markv.nl/blog/symmetric-encryption-in-rust
    #[test]
    fn demo() {
        // Key must be 32 bytes for Aes256. It should probably be the hashed
        // version of the input key, so is not limited to printable ascii.
        // SecVec has several advantages in preventing the key from leaking.
        let key = SecVec::from(b"RvzQW3Mwrc!_y5-DpPZl8rP3,=HsD1,!".to_vec());

        // The initialization vector (like salt or nonce) must be 16 bytes for
        // this block size. It could be generated using a secure randon generator,
        // and should be different each time. It is not a secret.
        let iv = vec![
            89, 63, 254, 34, 209, 155, 236, 158, 195, 104, 11, 16, 240, 4, 26, 76,
        ];

        // This is the data that is to be encrypted.
        let plaintext: Vec<u8> = b"Hello world! This is the secret text...".to_vec();

        // Encryption.
        // Fails if the key or iv are the wrong length, so it is safe to unwrap
        // as we have the correct lengths. Key length depends on algorithm, iv length
        // depends on the block size. If it's not documented, experiment with 16 or 32.
        let cipher = Aes256Cbc::new_var(key.unsecure(), &iv).unwrap();
        let ciphertext = cipher.encrypt_vec(&plaintext);

        // Check that it worked.
        assert_eq!(
            &ciphertext,
            &vec![
                216, 56, 166, 254, 171, 163, 243, 167, 235, 179, 189, 132, 0, 202, 44, 73, 10, 68,
                229, 90, 69, 212, 24, 22, 87, 109, 34, 92, 254, 136, 141, 154, 57, 189, 176, 221,
                140, 8, 114, 141, 103, 248, 108, 182, 247, 156, 113, 127,
            ]
        );

        // Decryption.
        let cipher = Aes256Cbc::new_var(key.unsecure(), &iv).unwrap();
        let decrypted_ciphertext = cipher.decrypt_vec(&ciphertext).unwrap();

        // Check that we got the original input back.
        assert_eq!(decrypted_ciphertext, plaintext);
    }
}
