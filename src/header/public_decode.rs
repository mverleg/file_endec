use ::std::collections::HashMap;
use ::std::io::BufRead;
use ::std::str::FromStr;

use ::semver::Version;

use crate::files::Checksum;
use crate::header::decode_util::{HeaderErr, read_header_keys, skip_header};
use crate::header::PUB_HEADER_CHECKSUM_MARKER;
use crate::header::PUB_HEADER_MARKER;
use crate::header::PUB_HEADER_META_DATA_MARKER;
use crate::header::PUB_HEADER_OPTION_MARKER;
use crate::header::PUB_HEADER_PURE_DATA_MARKER;
use crate::header::PUB_HEADER_SALT_MARKER;
use crate::header::PUB_HEADER_VERSION_MARKER;
use crate::header::PublicHeader;
use crate::key::salt::Salt;
use crate::util::errors::add_err;
use crate::util::FedResult;
use crate::util::option::{EncOption, EncOptionSet};

fn parse_version(header_data: &HashMap<String, String>, verbose: bool) -> FedResult<Version> {
    let version_str = header_data.get(PUB_HEADER_VERSION_MARKER)
        .ok_or("could not find the version in the file header".to_owned())?;
    match Version::parse(version_str) {
        Ok(version) => Ok(version),
        Err(err) => Err(add_err(
            format!("could not determine the version of fileenc that encrypted this file; got {} which is invalid", version_str),
            verbose,
            err,
        )),
    }
}

fn parse_options(header_data: &HashMap<String, String>, verbose: bool) -> FedResult<EncOptionSet> {
    let options_str = match header_data.get(PUB_HEADER_OPTION_MARKER) {
        Some(val) => val,
        None => return Ok(EncOptionSet::empty()),
    };
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

fn parse_salt(header_data: &HashMap<String, String>, verbose: bool) -> FedResult<Salt> {
    let salt_str = header_data.get(PUB_HEADER_SALT_MARKER)
        .ok_or("could not find the salt in the file header".to_owned())?;
    Salt::parse_base64(salt_str, verbose)
}

fn parse_checksum(header_data: &HashMap<String, String>) -> FedResult<Checksum> {
    let checksum_str = header_data.get(PUB_HEADER_CHECKSUM_MARKER)
        .ok_or("could not find the checksum in the file header".to_owned())?;
    Checksum::parse(checksum_str)
}

//TODO @mark: include filename in error at caller?
pub fn parse_public_header<R: BufRead>(reader: &mut R, verbose: bool) -> FedResult<PublicHeader> {

    let (_, header_data) = match read_header_keys(reader, Some(PUB_HEADER_MARKER), &[PUB_HEADER_PURE_DATA_MARKER, PUB_HEADER_META_DATA_MARKER]) {
        Ok(map) => map,
        Err(err) => return Err(if verbose {
            match err {
                HeaderErr::NoStartMarker => format!("did not recognize encryption header (expected '{}'); was this file really encrypted with fileenc?", PUB_HEADER_MARKER),
                HeaderErr::NoEndMarker => format!("could not find the end of the file header ('{}' or '{}'); has the file header been corrupted?", PUB_HEADER_PURE_DATA_MARKER, PUB_HEADER_META_DATA_MARKER),
                HeaderErr::HeaderSyntax(line) => format!("part of the file header could not be parsed because it did not have the expected format (found '{}')", &line),
                HeaderErr::ReadError => format!("the file header could not be read; perhaps the file was not accessible, or the file header has been corrupted"),
            }
        } else {
            match err {
                HeaderErr::NoStartMarker => format!("did not recognize encryption header; was this file really encrypted with fileenc?"),
                HeaderErr::NoEndMarker => format!("could not find the end of the file header; has the file header been corrupted?"),
                HeaderErr::HeaderSyntax(_) => format!("part of the file header could not be parsed because it did not have the expected format"),
                HeaderErr::ReadError => format!("the file header could not be read; perhaps the file was not accessible, or the file header has been corrupted"),
            }
        }),
    };

    let version = parse_version(&header_data, verbose)?;
    let options = parse_options(&header_data, verbose)?;
    let salt = parse_salt(&header_data, verbose)?;
    let checksum = parse_checksum(&header_data)?;
    Ok(PublicHeader::new(version, salt, checksum, options))
}

pub fn skip_public_header<R: BufRead>(reader: &mut R) -> FedResult<()> {
    skip_header(reader, &[PUB_HEADER_META_DATA_MARKER, PUB_HEADER_PURE_DATA_MARKER])
        .map_err(|_| "failed to skip past the header while reading file; possibly the header has been corrupted".to_string())
}

#[cfg(test)]
mod tests {
    use ::std::io::BufReader;
    use ::std::io::Read;

    use ::semver::Version;

    use crate::files::Checksum;
    use crate::header::public_decode::skip_public_header;
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
        skip_public_header(&mut reader).unwrap();
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
        );
        let mut buf = input.as_bytes();
        let header = parse_public_header(&mut buf, false).unwrap();
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
        );
        let mut buf = input.as_bytes();
        let header = parse_public_header(&mut buf, true).unwrap();
        assert_eq!(expected, header);
    }
}
