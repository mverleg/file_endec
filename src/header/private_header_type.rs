use crate::key::Salt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateHeader {
    // The original filename without directory path, with extension.
    filename: String,
    // Linux-like permissions in octal, e.g. "754" for "rwxrw.r..".
    permissions: Option<u32>,
    // Created and modified timestamps in nanoseconds.
    created_ns: Option<u128>,
    changed_ns: Option<u128>,
    accessed_ns: Option<u128>,
    // Original filesize in bytes.
    size: u64,
    // Secret seed for values like checksum. This prevents an attacker from verifying whether
    // an encrypted file contains a specific file that the attacker has access to.
    //TODO @mark: make sure pepper influences the checksum
    pepper: Salt,
    // Padding bytes length to obfuscate header size.
    //TODO @mark: padding data must not be very compressible, but should be deterministic
    padding_len: u16,
}

impl PrivateHeader {
    pub fn new(
        filename: String,
        permissions: Option<u32>,
        created_ns: Option<u128>,
        changed_ns: Option<u128>,
        accessed_ns: Option<u128>,
        size: u64,
        pepper: Salt,
        padding_len: u16,
    ) -> Self {
        debug_assert!(padding_len <= 1024); // implementation detail in padding data generation
        assert!(!filename.contains('\n'));
        PrivateHeader {
            filename,
            permissions,
            created_ns,
            changed_ns,
            accessed_ns,
            size,
            pepper,
            padding_len,
        }
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn permissions(&self) -> Option<u32> {
        self.permissions
    }

    pub fn created_ns(&self) -> Option<u128> {
        self.created_ns
    }

    pub fn changed_ns(&self) -> Option<u128> {
        self.changed_ns
    }

    pub fn accessed_ns(&self) -> Option<u128> {
        self.accessed_ns
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn pepper(&self) -> &Salt {
        &self.pepper
    }

    pub fn padding_len(&self) -> u16 {
        self.padding_len
    }
}

pub const PRIV_HEADER_FILENAME: &str = "name";
pub const PRIV_HEADER_PERMISSIONS: &str = "perm";
pub const PRIV_HEADER_CREATED: &str = "crt";
pub const PRIV_HEADER_MODIFIED: &str = "cng";
pub const PRIV_HEADER_ACCESSED: &str = "acs";
pub const PRIV_HEADER_SIZE: &str = "sz";
pub const PRIV_HEADER_PEPPER: &str = "pepr";
pub const PRIV_HEADER_PADDING: &str = "pad";
pub const PRIV_HEADER_DATA: &str = "enc:";
