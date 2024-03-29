use ::std::io::Write;

use ::semver::Version;

use crate::files::Checksum;
use crate::header::encode_util::write_line;
use crate::header::public_header_type::PUB_HEADER_META_DATA_MARKER;
use crate::header::public_header_type::PUB_HEADER_OPTION_MARKER;
use crate::header::PublicHeader;
use crate::header::PUB_HEADER_MARKER;
use crate::header::PUB_HEADER_SALT_MARKER;
use crate::header::PUB_HEADER_VERSION_MARKER;
use crate::header::{PUB_HEADER_CHECKSUM_MARKER, PUB_HEADER_PRIVATE_HEADER_META_MARKER};
use crate::key::salt::Salt;
use crate::util::base::u64_to_small_str;
use crate::util::version::version_has_options_meta;
use crate::util::FedResult;
use crate::EncOptionSet;

fn write_marker(writer: &mut impl Write, verbose: bool) -> FedResult<()> {
    write_line(writer, PUB_HEADER_MARKER, None, verbose)
}

fn write_version(writer: &mut impl Write, version: &Version, verbose: bool) -> FedResult<()> {
    let version_str = format!("{}.{}.{}", version.major, version.minor, version.patch);
    write_line(
        writer,
        PUB_HEADER_VERSION_MARKER,
        Some(&version_str),
        verbose,
    )
}

fn write_options(writer: &mut impl Write, options: &EncOptionSet, verbose: bool) -> FedResult<()> {
    if options.len() == 0 {
        return Ok(());
    }
    let options_txt = options
        .iter()
        .map(|opt| opt.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    write_line(
        writer,
        PUB_HEADER_OPTION_MARKER,
        Some(&options_txt),
        verbose,
    )
}

fn write_salt(writer: &mut impl Write, salt: &Salt, verbose: bool) -> FedResult<()> {
    let salt_str = salt.as_base64();
    write_line(writer, PUB_HEADER_SALT_MARKER, Some(&salt_str), verbose)
}

fn write_checksum(writer: &mut impl Write, checksum: &Checksum, verbose: bool) -> FedResult<()> {
    write_line(
        writer,
        PUB_HEADER_CHECKSUM_MARKER,
        Some(&format!("{}", checksum)),
        verbose,
    )
}

fn write_private_header_meta(
    writer: &mut impl Write,
    length: u64,
    checksum: &Checksum,
    verbose: bool,
) -> FedResult<()> {
    let value = format!("{} {}", u64_to_small_str(length), checksum);
    write_line(
        writer,
        PUB_HEADER_PRIVATE_HEADER_META_MARKER,
        Some(&value),
        verbose,
    )
}

pub fn write_public_header(
    writer: &mut impl Write,
    header: &PublicHeader,
    verbose: bool,
) -> FedResult<()> {
    write_marker(writer, verbose)?;
    write_version(writer, header.version(), verbose)?;
    if version_has_options_meta(header.version()) {
        write_options(writer, header.options(), verbose)?;
    }
    write_salt(writer, header.salt(), verbose)?;
    write_checksum(writer, header.checksum(), verbose)?;
    if let Some((length, checksum)) = header.private_header() {
        write_private_header_meta(writer, *length, checksum, verbose)?;
    }
    write_line(writer, PUB_HEADER_META_DATA_MARKER, None, verbose)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use ::std::str::from_utf8;

    use ::semver::Version;

    use crate::files::Checksum;
    use crate::header::PublicHeader;
    use crate::key::salt::Salt;
    use crate::util::option::EncOptionSet;

    use super::write_public_header;

    #[test]
    fn write_vanilla() {
        let version = Version::parse("1.1.0").unwrap();
        let header = PublicHeader::new(
            version,
            Salt::fixed_for_test(1),
            Checksum::fixed_for_test(vec![2]),
            EncOptionSet::empty(),
            (20, Checksum::fixed_for_test(vec![10, 20, 30])),
        );
        let mut buf: Vec<u8> = Vec::new();
        write_public_header(&mut buf, &header, true).unwrap();
        let expected =
            "github.com/mverleg/file_endec\0\nv 1.1.0\nsalt AQAAAAAAAAABAAAAAAAAAAEAAAAAAAAAAQAAAAAAAAABAAAAAAAAAAEAAAAAAAAAAQAAAAAAAAABAAAAAAAAAA\ncheck xx_sha256 Ag\nprv U xx_sha256 ChQe\nmeta1+data:\n";
        assert_eq!(expected, from_utf8(&buf).unwrap());
    }

    #[test]
    fn write_options() {
        let version = Version::parse("1.1.0").unwrap();
        let header = PublicHeader::new(
            version,
            Salt::fixed_for_test(123_456_789_123_456_789),
            Checksum::fixed_for_test(vec![0, 5, 0, 5, 0, 5, 0, 5, 0, 5, 0, 5]),
            EncOptionSet::all_for_test(),
            (20, Checksum::fixed_for_test(vec![10, 100])),
        );
        let mut buf: Vec<u8> = Vec::new();
        write_public_header(&mut buf, &header, true).unwrap();
        let expected = "github.com/mverleg/file_endec\0\nv 1.1.0\nopts fast hide-meta pad-size\nsalt FV_QrEubtgEVX9CsS5u2ARVf0KxLm7YBFV_QrEubtgEVX9CsS5u2ARVf0KxLm7YBFV_QrEubtgEVX9CsS5u2AQ\ncheck xx_sha256 AAUABQAFAAUABQAF\nprv U xx_sha256 CmQ\nmeta1+data:\n";
        assert_eq!(expected, from_utf8(&buf).unwrap());
    }
}
