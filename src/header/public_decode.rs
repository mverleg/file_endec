use ::std::io::BufRead;
use ::std::str::FromStr;

use ::semver::Version;

use crate::files::Checksum;
use crate::header::PublicHeader;
use crate::header::PUB_HEADER_CHECKSUM_MARKER;
use crate::header::PUB_HEADER_PURE_DATA_MARKER;
use crate::header::PUB_HEADER_MARKER;
use crate::header::PUB_HEADER_SALT_MARKER;
use crate::header::PUB_HEADER_VERSION_MARKER;
use crate::header::public_header_type::{PUB_HEADER_OPTION_MARKER, PUB_HEADER_META_DATA_MARKER};
use crate::key::salt::Salt;
use crate::util::errors::add_err;
use crate::util::FedResult;
use crate::util::option::{EncOption, EncOptionSet};
use crate::util::version::version_has_options_meta;

fn read_line(reader: &mut dyn BufRead, line: &mut String, verbose: bool) -> FedResult<()> {
    line.clear();
    let res = reader.read_line(line);
    if let Err(err) = res {
        return Err(add_err("could not read file", verbose, err));
    }
    line.pop();
    Ok(())
}

fn check_prefix<'a>(line: &'a str, prefix: &str, verbose: bool) -> FedResult<&'a str> {
    if line.starts_with(prefix) {
        Ok(&line[prefix.len()..])
    } else {
        Err(if verbose {
            "encryption header was incorrect".to_owned()
        } else {
            format!(
                "encryption header was incorrect (expected '{}', but it was not found)",
                prefix
            )
        })
    }
}

fn parse_marker(reader: &mut dyn BufRead, line: &mut String, verbose: bool) -> FedResult<()> {
    read_line(reader, line, verbose)?;
    if PUB_HEADER_MARKER != line {
        return Err(if verbose {
            format!("did not recognize encryption header (expected '{}', got '{}'); was this file really encrypted with fileenc?", PUB_HEADER_MARKER, line)
        } else {
            "did not recognize encryption header; was this file really encrypted with fileenc?"
                .to_owned()
        });
    }
    Ok(())
}

fn parse_version(reader: &mut dyn BufRead, line: &mut String, verbose: bool) -> FedResult<Version> {
    read_line(reader, line, verbose)?;
    let version_str = check_prefix(line, PUB_HEADER_VERSION_MARKER, verbose)?;
    match Version::parse(version_str) {
        Ok(version) => Ok(version),
        Err(err) => Err(add_err(
            format!(
                "could not determine the version \
            of fileenc that encrypted this file; got {} which is invalid",
                version_str
            ),
            verbose,
            err,
        )),
    }
}

fn parse_options(reader: &mut dyn BufRead, line: &mut String, verbose: bool) -> FedResult<EncOptionSet> {
    read_line(reader, line, verbose)?;
    let options_str = check_prefix(line, PUB_HEADER_OPTION_MARKER, verbose)?;
    let mut option_vec = vec![];
    for option_str in options_str.split_whitespace() {
        match EncOption::from_str(option_str) {
            Ok(option) => option_vec.push(option),
            Err(err) => return Err(add_err(
                format!("could not determine the options of fileenc that encrypted this file (got {} which is unknown); maybe it was encrypted with a newer version?", option_str),
                verbose,
                err,
            )),
        }
    }
    let option_count = option_vec.len();
    let options: EncOptionSet = option_vec.into();
    if options.len() != option_count {
        return Err(add_err(
            format!("there were duplicate encryption options in the file header; it is possible the header has been meddled with"),
            verbose,
            format!("found {}", options_str),
        ));
    }
    Ok(options)
}

fn parse_salt(reader: &mut dyn BufRead, line: &mut String, verbose: bool) -> FedResult<Salt> {
    read_line(reader, line, verbose)?;
    let salt_str = check_prefix(line, PUB_HEADER_SALT_MARKER, verbose)?;
    Salt::parse_base64(salt_str, verbose)
}

fn parse_checksum(
    reader: &mut dyn BufRead,
    line: &mut String,
    verbose: bool,
) -> FedResult<Checksum> {
    read_line(reader, line, verbose)?;
    let checksum_str = check_prefix(line, PUB_HEADER_CHECKSUM_MARKER, verbose)?;
    Checksum::parse(checksum_str)
}

pub fn parse_public_header<R: BufRead>(reader: &mut R, verbose: bool) -> FedResult<PublicHeader> {
    let mut line = String::new();
    parse_marker(reader, &mut line, verbose)?;
    let version = parse_version(reader, &mut line, verbose)?;
    let has_options = version_has_options_meta(&version);
    let options = if has_options {
        parse_options(reader, &mut line, verbose)?
    } else {
        EncOptionSet::empty()
    };
    let salt = parse_salt(reader, &mut line, verbose)?;
    let checksum = parse_checksum(reader, &mut line, verbose)?;
    read_line(reader, &mut line, verbose)?;
    if has_options {
        check_prefix(&line, PUB_HEADER_META_DATA_MARKER, verbose).unwrap();
    } else {
        check_prefix(&line, PUB_HEADER_PURE_DATA_MARKER, verbose).unwrap();
    }
    PublicHeader::new(version, salt, checksum, options)
}

pub fn skip_header<R: BufRead>(reader: &mut R, verbose: bool) -> FedResult<()> {
    let mut line = String::new();
    while !line.starts_with(PUB_HEADER_META_DATA_MARKER) && !line.starts_with(PUB_HEADER_PURE_DATA_MARKER) {
        read_line(reader, &mut line, verbose)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use ::std::io::BufReader;
    use ::std::io::Read;

    use ::semver::Version;

    use crate::files::Checksum;
    use crate::header::public_decode::skip_header;
    use crate::header::PublicHeader;
    use crate::key::salt::Salt;
    use crate::util::option::EncOptionSet;

    use super::parse_public_header;

    #[test]
    fn stop_read_after_header() {
        let _version = Version::parse("1.0.0").unwrap();
        let input =
            "github.com/mverleg/file_endec\0\nv 1.0.0\nsalt AQAAAAAAAAABAAAAAAAAAAEAAAAAAAAAAQAAAAAAAAABAAAAAAAAAAEAAAAAAAAAAQAAAAAAAAABAAAAAAAAAA\
            \ncheck xx_sha256 Ag\ndata:\nthis is the data and should not be read!\nthe end of the data";
        let mut reader = BufReader::new(input.as_bytes());
        parse_public_header(&mut reader, true).unwrap();
        let mut remainder = vec![];
        reader.read_to_end(&mut remainder).unwrap();
        let expected = "this is the data and should not be read!\nthe end of the data"
            .as_bytes()
            .to_owned();
        assert_eq!(expected, remainder);
    }

    #[test]
    fn skip_header_position() {
        let _version = Version::parse("1.0.0").unwrap();
        let input =
            "github.com/mverleg/file_endec\0\nv 1.0.0\nsalt AQAAAAAAAAABAAAAAAAAAAEAAAAAAAAAAQAAAAAAAAABAAAAAAAAAAEAAAAAAAAAAQAAAAAAAAABAAAAAAAAAA\
            \ncheck xx_sha256 Ag\ndata:\nthis is the data and should not be read!\nthe end of the data";
        let mut reader = BufReader::new(input.as_bytes());
        skip_header(&mut reader, true).unwrap();
        let mut remainder = vec![];
        reader.read_to_end(&mut remainder).unwrap();
        let expected = "this is the data and should not be read!\nthe end of the data"
            .as_bytes()
            .to_owned();
        assert_eq!(expected, remainder);
    }

    #[test]
    fn read_v1_0_0_one() {
        let version = Version::parse("1.0.0").unwrap();
        let input =
            "github.com/mverleg/file_endec\0\nv 1.0.0\nsalt AQAAAAAAAAABAAAAAAAAAAEAAAAAAAAAAQAAAAAAAAABAAAAAAAAAAEAAAAAAAAAAQAAAAAAAAABAAAAAAAAAA\ncheck xx_sha256 Ag\ndata:\n";
        let expected = PublicHeader::new(
            version,
            Salt::fixed_for_test(1),
            Checksum::fixed_for_test(vec![2]),
            EncOptionSet::empty(),  // always empty for v1.0
        )
        .unwrap();
        let mut buf = input.as_bytes();
        let header = parse_public_header(&mut buf, true).unwrap();
        assert_eq!(expected, header);
    }

    #[test]
    fn read_v1_0_0_two() {
        let version = Version::parse("1.0.0").unwrap();
        let input = "github.com/mverleg/file_endec\0\nv 1.0.0\nsalt FV_QrEubtgEVX9CsS5u2ARVf0KxLm7YBFV_QrEubtgEVX9CsS5u2ARVf0KxLm7YBFV_QrEubtgEVX9CsS5u2AQ\ncheck xx_sha256 AAUABQAFAAUABQAF\ndata:\n";
        let expected = PublicHeader::new(
            version,
            Salt::fixed_for_test(123_456_789_123_456_789),
            Checksum::fixed_for_test(vec![0, 5, 0, 5, 0, 5, 0, 5, 0, 5, 0, 5]),
            EncOptionSet::empty(),  // always empty for v1.0
        )
        .unwrap();
        let mut buf = input.as_bytes();
        let header = parse_public_header(&mut buf, true).unwrap();
        assert_eq!(expected, header);
    }
}
