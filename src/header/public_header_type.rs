use ::semver::Version;

use crate::files::Checksum;
use crate::key::Salt;
use crate::EncOptionSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicHeader {
    version: Version,
    salt: Salt,
    // Data checksum only BEFORE v1.1 (from 1.1 moved to private header)
    legacy_data_checksum: Option<Checksum>,
    // Options must be here because some of them affect private header decryption (like `fast`).
    options: EncOptionSet,
    // Length and checksum; required from v1.1
    private_header: Option<(u64, Checksum)>,
}

impl PublicHeader {
    pub fn new(version: Version, salt: Salt, options: EncOptionSet, private_header: (u64, Checksum)) -> Self {
        PublicHeader {
            version,
            salt,
            legacy_data_checksum: None,
            options,
            private_header: Some(private_header),
        }
    }

    /// Legacy version (which may not have private headers if it was before v1.1)
    pub fn legacy(version: Version, salt: Salt, data_checksum: Checksum) -> Self {
        PublicHeader {
            version,
            salt,
            legacy_data_checksum: Some(data_checksum),
            options: EncOptionSet::empty(),
            private_header: None,
        }
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn salt(&self) -> &Salt {
        &self.salt
    }

    pub fn options(&self) -> &EncOptionSet {
        &self.options
    }

    pub fn data_checksum(&self) -> &Option<Checksum> {
        &self.legacy_data_checksum
    }

    //TODO @mark: check checksum
    pub fn private_header(&self) -> &Option<(u64, Checksum)> {
        &self.private_header
    }
}

pub const PUB_HEADER_MARKER: &str = "github.com/mverleg/file_endec\0";
pub const PUB_HEADER_VERSION_MARKER: &str = "v";
pub const PUB_HEADER_SALT_MARKER: &str = "salt";
pub const PUB_HEADER_CHECKSUM_MARKER: &str = "check";
pub const PUB_HEADER_OPTION_MARKER: &str = "opts";
pub const PUB_HEADER_PRIVATE_HEADER_META_MARKER: &str = "prv";
pub const PUB_HEADER_PURE_DATA_MARKER: &str = "data:";
pub const PUB_HEADER_META_DATA_MARKER: &str = "meta1+data:";
