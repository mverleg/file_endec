use ::lazy_static::lazy_static;
use ::semver::Version;

lazy_static! {
    static ref CURRENT_VERSION: Version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    static ref OPTIONS_INTORDUCED_IN_VERSION: Version = Version::parse("1.1.0").unwrap();
}

pub fn get_current_version() -> Version {
    CURRENT_VERSION.clone()
}

/// Whether two features were used in the given version:
/// * private header with metadata
/// * customization options
//TODO v2.0: this should always be true, remove code for false branch
pub fn version_has_options_meta(version: &Version) -> bool {
    version >= &*OPTIONS_INTORDUCED_IN_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimum_version() {
        assert!(*CURRENT_VERSION >= Version::parse("1.0.0").unwrap());
    }

    #[test]
    fn numbers_only() {
        assert_eq!(0, CURRENT_VERSION.build.len());
        assert_eq!(0, CURRENT_VERSION.pre.len());
    }
}
