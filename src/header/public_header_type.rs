use ::semver::Version;

use crate::files::Checksum;
use crate::key::Salt;
use crate::util::FedResult;
use crate::util::option::EncOptionSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicHeader {
    version: Version,
    salt: Salt,
    checksum: Checksum,
    options: EncOptionSet,
}

impl PublicHeader {
    pub fn new(version: Version, salt: Salt, checksum: Checksum, options: EncOptionSet,) -> FedResult<Self> {
        Ok(PublicHeader {
            version,
            salt,
            checksum,
            options,
        })
    }

    pub fn version(&self) -> &Version {
        &self.version
    }
    pub fn salt(&self) -> &Salt {
        &self.salt
    }
    pub fn checksum(&self) -> &Checksum {
        &self.checksum
    }
    pub fn options(&self) -> &EncOptionSet {
        &self.options
    }
}

pub const HEADER_MARKER: &str = "github.com/mverleg/file_endec\0";
pub const HEADER_VERSION_MARKER: &str = "v ";
pub const HEADER_SALT_MARKER: &str = "salt ";
pub const HEADER_CHECKSUM_MARKER: &str = "check ";
pub const HEADER_OPTION_MARKER: &str = "opts ";
pub const HEADER_PURE_DATA_MARKER: &str = "data:";
pub const HEADER_META_DATA_MARKER: &str = "meta1+data:";