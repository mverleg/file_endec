use ::semver::Version;

use crate::files::Checksum;
use crate::key::Salt;
use crate::util::FedResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    version: Version,
    salt: Salt,
    checksum: Checksum,
}

impl Header {
    pub fn new(version: Version, salt: Salt, checksum: Checksum) -> FedResult<Self> {
        Ok(Header {
            version,
            salt,
            checksum,
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
}

pub const HEADER_MARKER: &str = "github.com/mverleg/file_endec";
pub const HEADER_VERSION_MARKER: &str = "v ";
pub const HEADER_SALT_MARKER: &str = "salt ";
pub const HEADER_CHECKSUM_MARKER: &str = "check ";
pub const HEADER_DATA_MARKER: &str = "data:";
