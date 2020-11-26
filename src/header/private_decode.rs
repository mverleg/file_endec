use ::std::collections::HashMap;
use ::std::io::BufRead;
use ::std::str::FromStr;

use ::semver::Version;

use crate::files::Checksum;
use crate::header::decode_util::HeaderErr;
use crate::header::decode_util::read_header_keys;
use crate::header::decode_util::skip_header;
use crate::header::private_header_type::{PRIV_HEADER_DATA, PRIV_HEADER_FILENAME, PRIV_HEADER_CREATED, PRIV_HEADER_MODIFIED, PRIV_HEADER_ACCESSED};
use crate::header::private_header_type::PrivateHeader;
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
use crate::util::option::EncOption;
use crate::util::option::EncOptionSet;
use crate::util::base::small_str_to_u128;

fn parse_filename(header_data: &HashMap<String, String>) -> FedResult<String> {
    header_data.get(PRIV_HEADER_FILENAME).cloned()
        .ok_or("could not find the original filename in the file header".to_owned())
}

fn parse_sizes(header_data: &HashMap<String, String>) -> FedResult<(Option<u128>, Option<u128>, Option<u128>)> {
    Ok((
        header_data.get(PRIV_HEADER_CREATED).map(|ts| small_str_to_u128(ts)).unwrap(),
        header_data.get(PRIV_HEADER_MODIFIED).map(|ts| small_str_to_u128(ts)).unwrap(),
        header_data.get(PRIV_HEADER_ACCESSED).map(|ts| small_str_to_u128(ts)).unwrap(),
    ))
}

//TODO @mark: include filename in error at caller?
pub fn parse_private_header<R: BufRead>(reader: &mut R, verbose: bool) -> FedResult<PrivateHeader> {

    let header_data = match read_header_keys(reader, None, &[PRIV_HEADER_DATA]) {
        Ok(map) => map,
        Err(err) => return Err(match err {
            HeaderErr::NoStartMarker => unreachable!(),
            HeaderErr::NoEndMarker => format!("could not find the end of the private file header inside encrypted block; has the file been corrupted?"),
            HeaderErr::HeaderSyntax(_) => format!("part of the private file header inside encrypted block could not be parsed because it did not have the expected format"),
            HeaderErr::ReadError => format!("the private file header inside encrypted block could not be read; perhaps the file was not accessible, or the file header has been corrupted"),
        }),
    };

    let filename = parse_filename(&header_data)?;
    let (created_ns, changed_ns, accessed_ns) = parse_sizes(&header_data)?;
    Ok(PrivateHeader::new(
        filename,
        permissions,
        created_ns,
        changed_ns,
        accessed_ns,
        size,
    ))
}

pub fn skip_public_header<R: BufRead>(reader: &mut R) -> FedResult<()> {
    skip_header(reader, &[PUB_HEADER_META_DATA_MARKER, PUB_HEADER_PURE_DATA_MARKER])
        .map_err(|_| "failed to skip past the header while reading file; possibly the header has been corrupted".to_string())
}

#[cfg(test)]
mod tests {
    //TODO @mark: update all these tests

    use ::std::io::BufReader;
    use ::std::io::Read;

    use ::semver::Version;

    use crate::files::Checksum;
    use crate::header::public_decode::skip_public_header;
    use crate::header::PublicHeader;
    use crate::key::salt::Salt;
    use crate::util::option::EncOptionSet;

    // #[test]
    // fn stop_read_after_header() {
    //     let _version = Version::parse("1.0.0").unwrap();
    //     let input =
    //         "github.com/mverleg/file_endec\0\nv 1.0.0\nsalt AQAAAAAAAAABAAAAAAAAAAEAAAAAAAAAAQAAAAAAAAABAAAAAAAAAAEAAAAAAAAAAQAAAAAAAAABAAAAAAAAAA\
    //         \ncheck xx_sha256 Ag\ndata:\nthis is the data and should not be read!\nthe end of the data";
    //     let mut reader = BufReader::new(input.as_bytes());
    //     parse_public_header(&mut reader, true).unwrap();
    //     let mut remainder = vec![];
    //     reader.read_to_end(&mut remainder).unwrap();
    //     let expected = "this is the data and should not be read!\nthe end of the data"
    //         .as_bytes()
    //         .to_owned();
    //     assert_eq!(expected, remainder);
    // }
    //
    // #[test]
    // fn skip_header_position() {
    //     let _version = Version::parse("1.0.0").unwrap();
    //     let input =
    //         "github.com/mverleg/file_endec\0\nv 1.0.0\nsalt AQAAAAAAAAABAAAAAAAAAAEAAAAAAAAAAQAAAAAAAAABAAAAAAAAAAEAAAAAAAAAAQAAAAAAAAABAAAAAAAAAA\
    //         \ncheck xx_sha256 Ag\ndata:\nthis is the data and should not be read!\nthe end of the data";
    //     let mut reader = BufReader::new(input.as_bytes());
    //     skip_public_header(&mut reader).unwrap();
    //     let mut remainder = vec![];
    //     reader.read_to_end(&mut remainder).unwrap();
    //     let expected = "this is the data and should not be read!\nthe end of the data"
    //         .as_bytes()
    //         .to_owned();
    //     assert_eq!(expected, remainder);
    // }
    //
    // #[test]
    // fn read_v1_0_0_one() {
    //     let version = Version::parse("1.0.0").unwrap();
    //     let input =
    //         "github.com/mverleg/file_endec\0\nv 1.0.0\nsalt AQAAAAAAAAABAAAAAAAAAAEAAAAAAAAAAQAAAAAAAAABAAAAAAAAAAEAAAAAAAAAAQAAAAAAAAABAAAAAAAAAA\ncheck xx_sha256 Ag\ndata:\n";
    //     let expected = PublicHeader::new(
    //         version,
    //         Salt::fixed_for_test(1),
    //         Checksum::fixed_for_test(vec![2]),
    //         EncOptionSet::empty(),  // always empty for v1.0
    //     );
    //     let mut buf = input.as_bytes();
    //     let header = parse_public_header(&mut buf, false).unwrap();
    //     assert_eq!(expected, header);
    // }
    //
    // #[test]
    // fn read_v1_0_0_two() {
    //     let version = Version::parse("1.0.0").unwrap();
    //     let input = "github.com/mverleg/file_endec\0\nv 1.0.0\nsalt FV_QrEubtgEVX9CsS5u2ARVf0KxLm7YBFV_QrEubtgEVX9CsS5u2ARVf0KxLm7YBFV_QrEubtgEVX9CsS5u2AQ\ncheck xx_sha256 AAUABQAFAAUABQAF\ndata:\n";
    //     let expected = PublicHeader::new(
    //         version,
    //         Salt::fixed_for_test(123_456_789_123_456_789),
    //         Checksum::fixed_for_test(vec![0, 5, 0, 5, 0, 5, 0, 5, 0, 5, 0, 5]),
    //         EncOptionSet::empty(),  // always empty for v1.0
    //     );
    //     let mut buf = input.as_bytes();
    //     let header = parse_public_header(&mut buf, true).unwrap();
    //     assert_eq!(expected, header);
    // }
}
