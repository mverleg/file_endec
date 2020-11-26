use ::std::collections::HashMap;
use ::std::io::BufRead;

use crate::header::decode_util::HeaderErr;
use crate::header::decode_util::read_header_keys;
use crate::header::decode_util::skip_header;
use crate::header::private_header_type::{PRIV_HEADER_ACCESSED, PRIV_HEADER_CREATED, PRIV_HEADER_DATA, PRIV_HEADER_FILENAME, PRIV_HEADER_MODIFIED, PRIV_HEADER_PERMISSIONS, PRIV_HEADER_SIZE};
use crate::header::private_header_type::PrivateHeader;
use crate::header::PUB_HEADER_META_DATA_MARKER;
use crate::header::PUB_HEADER_PURE_DATA_MARKER;
use crate::util::base::small_str_to_u128;
use crate::util::base::small_str_to_u64;
use crate::util::FedResult;

fn parse_filename(header_data: &HashMap<String, String>) -> FedResult<String> {
    header_data.get(PRIV_HEADER_FILENAME).cloned()
        .ok_or("could not find the original filename in the private file header".to_owned())
}

fn parse_permissions(header_data: &HashMap<String, String>) -> FedResult<Option<u32>> {
    //TODO @mark: test parsing
    Ok(header_data.get(PRIV_HEADER_PERMISSIONS).map(|sz| u32::from_str_radix(sz, 8).unwrap()))
}

fn parse_sizes(header_data: &HashMap<String, String>) -> FedResult<(Option<u128>, Option<u128>, Option<u128>)> {
    Ok((
        header_data.get(PRIV_HEADER_CREATED).map(|ts| small_str_to_u128(ts).unwrap()),
        header_data.get(PRIV_HEADER_MODIFIED).map(|ts| small_str_to_u128(ts).unwrap()),
        header_data.get(PRIV_HEADER_ACCESSED).map(|ts| small_str_to_u128(ts).unwrap()),
    ))
}

fn parse_size(header_data: &HashMap<String, String>) -> FedResult<u64> {
    header_data.get(PRIV_HEADER_SIZE).map(|sz| small_str_to_u64(sz).unwrap())
        .ok_or("could not find the original file size in the private file header".to_owned())
}

//TODO @mark: include filename in error at caller?
pub fn parse_private_header<R: BufRead>(reader: &mut R) -> FedResult<PrivateHeader> {

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
    let permissions = parse_permissions(&header_data)?;
    let (created, changed, accessed) = parse_sizes(&header_data)?;
    let size = parse_size(&header_data)?;
    Ok(PrivateHeader::new(
        filename,
        permissions,
        created,
        changed,
        accessed,
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


    use crate::header::private_header_type::PrivateHeader;
    use crate::header::private_decode::parse_private_header;
    use crate::EncOptionSet;

    #[test]
    fn read_vanilla() {
        let mut txt = "enc:\n".as_bytes();
        let actual = parse_private_header(&mut txt).unwrap();
        let expected = PrivateHeader::new(
            "my_filename.ext".to_owned(),
            Some(0o754),
            Some(123_456_789_000),
            Some(987_654_321_000),
            Some(999_999_999_999),
            1024_000,
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn read_hide_meta_size() {
        let mut txt = "name my_filename.ext\nperm 754\ncrt Ax9lQnI\ncng NWzxOMo\nacs NiToP-_\nsz C4_A\nenc:\n".as_bytes();
        let actual = parse_private_header(&mut txt).unwrap();
        let expected = PrivateHeader::new(
            "my_filename.ext".to_owned(),
            Some(0o754),
            Some(123_456_789_000),
            Some(987_654_321_000),
            Some(999_999_999_999),
            1024_000,
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn read_hide_unsupported() {
        let mut txt = "name my_filename.ext\ncrt Ax9lQnI\ncng NWzxOMo\nsz C4_A\nenc:\n".as_bytes();
        let actual = parse_private_header(&mut txt).unwrap();
        let expected = PrivateHeader::new(
            "my_filename.ext".to_owned(),
            None,
            Some(123_456_789_000),
            Some(987_654_321_000),
            None,
            1024_000,
        );
        assert_eq!(actual, expected);
    }
}
