use ::std::error::Error;
use ::std::io::Write;

use ::semver::Version;

use crate::EncOptionSet;
use crate::files::Checksum;
use crate::header::private_header_type::{PRIV_HEADER_CHANGED, PRIV_HEADER_CREATED, PRIV_HEADER_DATA, PRIV_HEADER_FILENAME, PRIV_HEADER_PERMISSIONS, PRIV_HEADER_SIZE, PrivateHeader};
use crate::header::PUB_HEADER_CHECKSUM_MARKER;
use crate::header::PUB_HEADER_MARKER;
use crate::header::PUB_HEADER_SALT_MARKER;
use crate::header::PUB_HEADER_VERSION_MARKER;
use crate::header::public_header_type::{PUB_HEADER_META_DATA_MARKER, PUB_HEADER_OPTION_MARKER};
use crate::header::PublicHeader;
use crate::key::salt::Salt;
use crate::util::errors::add_err;
use crate::util::FedResult;
use crate::util::version::version_has_options_meta;

pub fn write_private_header(writer: &mut impl Write, header: &PrivateHeader, verbose: bool) -> FedResult<()> {
    write_line(writer, PRIV_HEADER_FILENAME, Some(header.name()), verbose)?;
    write_line(writer, PRIV_HEADER_PERMISSIONS, Some(header.permissions()), verbose)?;
    write_line(writer, PRIV_HEADER_CREATED, Some(header.created_ns()), verbose)?;
    write_line(writer, PRIV_HEADER_CHANGED, Some(header.changed_ns()), verbose)?;
    write_line(writer, PRIV_HEADER_SIZE, Some(header.size()), verbose)?;
    write_line(writer, PRIV_HEADER_DATA, None, verbose)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use ::std::str::from_utf8;

    use ::semver::Version;

    use crate::files::Checksum;
    use crate::header::PublicHeader;
    use crate::key::salt::Salt;
    use crate::util::option::EncOptionSet;

    use super::write_public_header;

    #[test]
    fn write_vanilla() {
        let header = PrivateHeader::new(
            "my_filename.ext".to_owned(),
            0o754,
            123_456_789_000,
            987_654_321_000,
            1024_000,
        );
        let mut buf: Vec<u8> = Vec::new();
        write_private_header(&mut buf, &header, true).unwrap();
        let expected =
            "github.com/mverleg/file_endec\0\nv 1.1.0\nsalt AQAAAAAAAAABAAAAAAAAAAEAAAAAAAAAAQAAAAAAAAABAAAAAAAAAAEAAAAAAAAAAQAAAAAAAAABAAAAAAAAAA\ncheck xx_sha256 Ag\nmeta1+data:\n";
        assert_eq!(expected, from_utf8(&buf).unwrap());
    }
}
