use crate::util::FedResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateHeader {
    // The original filename without directory path, with extension.
    filename: String,
    // Linux-like permissions in octal, e.g. "754" for "rwxrw.r..".
    permissions: u32,
    // Created and modified timestamps in nanoseconds.
    created_ns: u128,
    changed_ns: u128,
    // Original filesize in bytes
    size: u64,
}

impl PrivateHeader {
    pub fn new(filename: String, permissions: u32, created_ns: u128, changed_ns: u128, size: u64,) -> FedResult<Self> {
        Ok(PrivateHeader {
            filename,
            permissions,
            created_ns,
            changed_ns,
            size,
        })
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn permissions(&self) -> u32 {
        self.permissions
    }

    pub fn created_ns(&self) -> u128 {
        self.created_ns
    }

    pub fn changed_ns(&self) -> u128 {
        self.changed_ns
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}

pub const PRIV_HEADER_FILENAME: &str = "name";
pub const PRIV_HEADER_PERMISSIONS: &str = "perm";
pub const PRIV_HEADER_CREATED: &str = "crt";
pub const PRIV_HEADER_CHANGED: &str = "cng";
pub const PRIV_HEADER_SIZE: &str = "sz";
pub const PRIV_HEADER_DATA: &str = "data:";
