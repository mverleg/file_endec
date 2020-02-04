
pub mod strategy;
pub mod types;
pub mod encode;
pub mod decode;

pub use decode::parse_header;
pub use encode::write_header;
pub use strategy::CompressionAlg;
pub use strategy::get_version_strategy;
pub use strategy::KeyHashAlg;
pub use strategy::Strategy;
pub use strategy::SymmetricEncryptionAlg;
pub use types::Header;
pub use types::HEADER_CHECKSUM_MARKER;
pub use types::HEADER_DATA_MARKER;
pub use types::HEADER_MARKER;
pub use types::HEADER_SALT_MARKER;
pub use types::HEADER_VERSION_MARKER;
pub use types::Salt;

