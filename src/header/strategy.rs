use ::std::fmt;
use ::std::fmt::Formatter;

use ::lazy_static::lazy_static;
use ::semver::Version;

use crate::util::FedResult;
use crate::util::option::{EncOptionSet, EncOption};
use crate::util::version::get_current_version;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbosity {
    Quiet,
    Normal,
    Debug,
}

impl Default for Verbosity {
    fn default() -> Self {
        Verbosity::Normal
    }
}

impl Verbosity {
    pub fn debug(self) -> bool {
        Verbosity::Debug == self
    }

    pub fn quiet(self) -> bool {
        Verbosity::Quiet == self
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CompressionAlg {
    Brotli,
}

impl fmt::Display for CompressionAlg {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(match self {
            CompressionAlg::Brotli => "brotli",
        })
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum KeyHashAlg {
    BCrypt,
    Argon2i,
    Sha512,
}

impl fmt::Display for KeyHashAlg {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(match self {
            KeyHashAlg::BCrypt => "bcrypt",
            KeyHashAlg::Argon2i => "argon2i",
            KeyHashAlg::Sha512 => "sha512",
        })
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SymmetricEncryptionAlg {
    // Aes 256 with Iso7816 padding and cipher block chaining
    Aes256,
    // Twofish with Iso7816 padding and cipher block chaining
    Twofish,
}

impl fmt::Display for SymmetricEncryptionAlg {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(match self {
            SymmetricEncryptionAlg::Aes256 => "aes256",
            SymmetricEncryptionAlg::Twofish => "twofish",
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Strategy {
    pub stretch_count: u64,
    pub compression_algorithm: Option<CompressionAlg>,
    pub key_hash_algorithms: Vec<KeyHashAlg>,
    pub symmetric_algorithms: Vec<SymmetricEncryptionAlg>,
}

lazy_static! {
    static ref STRATEGY_1_0: Strategy = Strategy {
        stretch_count: 5,
        compression_algorithm: Some(CompressionAlg::Brotli),
        key_hash_algorithms: vec![KeyHashAlg::BCrypt, KeyHashAlg::Argon2i, KeyHashAlg::Sha512],
        symmetric_algorithms: vec![
            SymmetricEncryptionAlg::Aes256,
            SymmetricEncryptionAlg::Twofish
        ],
    };
    static ref STRATEGY_1_1_FAST: Strategy = Strategy {
        stretch_count: 2,
        compression_algorithm: Some(CompressionAlg::Brotli),
        key_hash_algorithms: vec![KeyHashAlg::Argon2i, KeyHashAlg::Sha512],
        symmetric_algorithms: vec![
            SymmetricEncryptionAlg::Aes256,
        ],
    };
}

/// Get the encryption strategy used for a specific code version.
pub fn get_version_strategy(version: &Version, options: &EncOptionSet, verbose: bool) -> FedResult<&'static Strategy> {
    // This should return the strategy for all old versions - don't delete any, just add new ones!
    if version < &Version::parse("1.0.0").unwrap() {
        return Err(if verbose {
            "non-existent version".to_owned()
        } else {
            format!("non-existent version {} (minimum is 1.0.0)", version)
        });
    }
    if options.has(EncOption::Fast) {
        Ok(&*STRATEGY_1_1_FAST)
    } else {
        Ok(&*STRATEGY_1_0)
    }
}

pub fn get_current_version_strategy(options: &EncOptionSet, verbose: bool) -> &'static Strategy {
    get_version_strategy(&get_current_version(), options, verbose).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn determine_strategy_1_0() {
        assert_eq!(&*STRATEGY_1_0, get_version_strategy(&Version::parse("1.0.0").unwrap(), &EncOptionSet::empty(), true).unwrap());
        assert_eq!(&*STRATEGY_1_0, get_version_strategy(&Version::parse("1.0.0").unwrap(), &EncOptionSet::empty(), false).unwrap());
    }

    #[test]
    fn determine_strategy_1_1() {
        assert_eq!(&*STRATEGY_1_0, get_version_strategy(&Version::parse("1.1.0").unwrap(), &EncOptionSet::empty(), true).unwrap());
        assert_eq!(&*STRATEGY_1_0, get_version_strategy(&Version::parse("1.1.0").unwrap(), &EncOptionSet::empty(), false).unwrap());
        assert_eq!(&*STRATEGY_1_1_FAST, get_version_strategy(&Version::parse("1.1.0").unwrap(), &EncOptionSet::all_for_test(), true).unwrap());
        assert_eq!(&*STRATEGY_1_1_FAST, get_version_strategy(&Version::parse("1.1.0").unwrap(), &EncOptionSet::all_for_test(), false).unwrap());
    }
}
