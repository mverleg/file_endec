use ::std::error::Error;
use ::std::io::Write;

use ::semver::Version;

use crate::files::Checksum;
use crate::header::Header;
use crate::header::Salt;
use crate::header::HEADER_CHECKSUM_MARKER;
use crate::header::HEADER_DATA_MARKER;
use crate::header::HEADER_MARKER;
use crate::header::HEADER_SALT_MARKER;
use crate::header::HEADER_VERSION_MARKER;
use crate::util::util::u64_to_base64str;
use crate::util::FedResult;

fn wrap_err(res: Result<usize, impl Error>, verbose: bool) -> FedResult<()> {
    if let Err(err) = res {
        Err(match verbose {
            true => "failed to write encryption header".to_owned(),
            false => format!("failed to write encryption header, reason: {}", err),
        })
    } else {
        Ok(())
    }
}

fn write_line(
    writer: &mut impl Write,
    prefix: &str,
    value: Option<String>,
    verbose: bool,
) -> FedResult<()> {
    wrap_err(writer.write(prefix.as_bytes()), verbose)?;
    if let Some(text) = value {
        wrap_err(writer.write(text.as_bytes()), verbose)?;
    }
    wrap_err(writer.write(b"\n"), verbose)?;
    Ok(())
}

fn write_marker(writer: &mut impl Write, verbose: bool) -> FedResult<()> {
    write_line(writer, HEADER_MARKER, None, verbose)
}

fn write_version(writer: &mut impl Write, version: &Version, verbose: bool) -> FedResult<()> {
    let version_str = format!("{}.{}.{}", version.major, version.minor, version.patch);
    write_line(writer, HEADER_VERSION_MARKER, Some(version_str), verbose)
}

fn write_salt(writer: &mut impl Write, salt: &Salt, verbose: bool) -> FedResult<()> {
    let salt_str = u64_to_base64str(salt.as_primitive());
    write_line(writer, HEADER_SALT_MARKER, Some(salt_str), verbose)
}

fn write_checksum(writer: &mut impl Write, checksum: &Checksum, verbose: bool) -> FedResult<()> {
    write_line(
        writer,
        HEADER_CHECKSUM_MARKER,
        Some(format!("{}", checksum)),
        verbose,
    )
}

pub fn write_header(writer: &mut impl Write, header: &Header, verbose: bool) -> FedResult<()> {
    write_marker(writer, verbose)?;
    write_version(writer, header.version(), verbose)?;
    write_salt(writer, header.salt(), verbose)?;
    write_checksum(writer, header.checksum(), verbose)?;
    write_line(writer, HEADER_DATA_MARKER, None, verbose)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use semver::Version;

    use crate::files::Checksum;
    use crate::header::Header;
    use crate::header::Salt;

    use super::write_header;

    #[test]
    fn write_v1_0_0_one() {
        let version = Version::parse("1.0.0").unwrap();
        let header = Header::new(
            version,
            Salt::new(1),
            Checksum::fixed_for_test(vec![2]),
            &true,
        )
        .unwrap();
        let mut buf: Vec<u8> = Vec::new();
        write_header(&mut buf, &header, true).unwrap();
        let expected =
            "github.com/mverleg/file_endec\nv 1.0.0\nsalt AQAAAAAAAAA\ncheck xx_sha256 Ag\ndata:\n";
        assert_eq!(expected, from_utf8(&buf).unwrap());
    }

    #[test]
    fn write_v1_0_0_two() {
        let version = Version::parse("1.0.0").unwrap();
        let header = Header::new(
            version,
            Salt::new(123_456_789),
            Checksum::fixed_for_test(vec![0, 5, 0, 5, 0, 5, 0, 5, 0, 5, 0, 5]),
            &true,
        )
        .unwrap();
        let mut buf: Vec<u8> = Vec::new();
        write_header(&mut buf, &header, true).unwrap();
        let expected = "github.com/mverleg/file_endec\nv 1.0.0\nsalt Fc1bBwAAAAA\ncheck xx_sha256 AAUABQAFAAUABQAF\ndata:\n";
        assert_eq!(expected, from_utf8(&buf).unwrap());
    }
}
