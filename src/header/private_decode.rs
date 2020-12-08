use ::std::collections::HashMap;
use ::std::io::BufRead;

use crate::header::decode_util::HeaderErr;
use crate::header::decode_util::read_header_keys;
use crate::header::private_header_type::{PRIV_HEADER_ACCESSED, PRIV_HEADER_CREATED, PRIV_HEADER_DATA, PRIV_HEADER_FILENAME, PRIV_HEADER_MODIFIED, PRIV_HEADER_PADDING, PRIV_HEADER_PEPPER, PRIV_HEADER_PERMISSIONS, PRIV_HEADER_SIZE};
use crate::header::private_header_type::PrivateHeader;
use crate::key::Salt;
use crate::util::base::small_str_to_u128;
use crate::util::base::small_str_to_u64;
use crate::util::FedResult;

fn parse_filename(header_data: &mut HashMap<String, String>) -> FedResult<String> {
    header_data.remove(PRIV_HEADER_FILENAME)
        .ok_or("could not find the original filename in the private file header".to_owned())
}

fn parse_permissions(header_data: &mut HashMap<String, String>) -> FedResult<Option<u32>> {
    //TODO @mark: test parsing
    Ok(header_data.remove(PRIV_HEADER_PERMISSIONS).map(|sz| u32::from_str_radix(&sz, 8).unwrap()))
}

fn parse_sizes(header_data: &mut HashMap<String, String>) -> FedResult<(Option<u128>, Option<u128>, Option<u128>)> {
    Ok((
        header_data.remove(PRIV_HEADER_CREATED).map(|ts| small_str_to_u128(&ts).unwrap()),
        header_data.remove(PRIV_HEADER_MODIFIED).map(|ts| small_str_to_u128(&ts).unwrap()),
        header_data.remove(PRIV_HEADER_ACCESSED).map(|ts| small_str_to_u128(&ts).unwrap()),
    ))
}

fn parse_size(header_data: &mut HashMap<String, String>) -> FedResult<u64> {
    header_data.remove(PRIV_HEADER_SIZE).map(|sz| small_str_to_u64(&sz).unwrap())
        .ok_or("could not find the original file size in the private file header".to_owned())
}

/// Pepper and padding are included to obfuscate metadata.
fn parse_obfuscation(header_data: &mut HashMap<String, String>) -> FedResult<(Salt, u16)> {
    let pepper = header_data.remove(PRIV_HEADER_PEPPER).map(|pepr| Salt::parse_base64(&pepr, false))
        .ok_or("could not find the pepper (private salt) in the private file header".to_owned())??;
    let padding_len = header_data.remove(PRIV_HEADER_PADDING).map(|pad| pad.len() as u16)
        .ok_or("could not find the padding in the private file header".to_owned())?;
    Ok((pepper, padding_len))
}

//TODO @mark: include filename in error at caller?
/// Parses the data in the private header and returns it, along with the index of the first byte after the header.
pub fn parse_private_header<R: BufRead>(reader: &mut R) -> FedResult<(usize, PrivateHeader)> {

    let (index, mut header_data) = match read_header_keys(reader, None, &[PRIV_HEADER_DATA]) {
        Ok(map) => map,
        Err(err) => return Err(match err {
            HeaderErr::NoStartMarker => unreachable!(),
            HeaderErr::NoEndMarker => format!("could not find the end of the private file header; has the file been corrupted?"),
            HeaderErr::HeaderSyntax(_) => format!("part of the private file header could not be parsed because it did not have the expected format"),
            HeaderErr::ReadError => format!("the private file header could not be read; perhaps the file was not accessible, or the file header has been corrupted"),
        }),
    };

    let filename = parse_filename(&mut header_data)?;
    let permissions = parse_permissions(&mut header_data)?;
    let (created, changed, accessed) = parse_sizes(&mut header_data)?;
    let size = parse_size(&mut header_data)?;
    let (pepper, padding_len) = parse_obfuscation(&mut header_data)?;

    if !header_data.is_empty() {
        let key_names = header_data.iter()
            .map(|(key, _)| key.as_str())
            .collect::<Vec<_>>().join("', '");
        eprintln!("encountered unknown private header keys '{}'; this may happen if the file is encrypted using a newer version of file_endec, or if the file is corrupt; ignoring this problem", key_names);
    }

    Ok((index, PrivateHeader::new(
        filename,
        permissions,
        created,
        changed,
        accessed,
        size,
        pepper,
        padding_len,
    )))
}

#[cfg(test)]
mod tests {
    use ::std::collections::HashMap;

    use crate::header::private_decode::{parse_permissions, parse_private_header};
    use crate::header::private_header_type::{PRIV_HEADER_PERMISSIONS, PrivateHeader};
    use crate::key::Salt;

    #[test]
    fn permissions() {
        let mut map = HashMap::new();
        map.insert(PRIV_HEADER_PERMISSIONS.to_owned(), "754".to_owned());
        let perms = parse_permissions(&mut map);
        assert_eq!(perms, Ok(Some(0o754)));
        assert!(map.is_empty());
    }

    #[test]
    fn read_vanilla() {
        let mut txt = "name my_filename.ext\nsz C4_A\npepr EiGaAAAAAAASIZoAAAAAABIhmgAAAAAAEiGaAAAAAAASIZoAAAAAABIhmgAAAAAAEiGaAAAAAAASIZoAAAAAAA\npad \nenc:\n".as_bytes();
        let (length, actual) = parse_private_header(&mut txt).unwrap();
        let expected = PrivateHeader::new(
            "my_filename.ext".to_owned(),
            None,
            None,
            None,
            None,
            1024_000,
            Salt::fixed_for_test(010_101_010),
            0,
        );
        assert_eq!(length, 131);
        assert_eq!(actual, expected);
    }

    #[test]
    fn read_hide_meta_size() {
        let mut txt = "name my_filename.ext\nperm 754\ncrt Ax9lQnI\ncng NWzxOMo\nacs NiToP-_\nsz C4_A\npepr zeO1DgAAAADN47UOAAAAAM3jtQ4AAAAAzeO1DgAAAADN47UOAAAAAM3jtQ4AAAAAzeO1DgAAAADN47UOAAAAAA\npad I(R#:Y:r$F\nenc:\n".as_bytes();
        let (length, actual) = parse_private_header(&mut txt).unwrap();
        let expected = PrivateHeader::new(
            "my_filename.ext".to_owned(),
            Some(0o754),
            Some(123_456_789_000),
            Some(987_654_321_000),
            Some(999_999_999_999),
            1024_000,
            Salt::fixed_for_test(246_801_357),
            10,
        );
        assert_eq!(length, 186);
        assert_eq!(actual, expected);
    }

    #[test]
    fn read_hide_unsupported() {
        let mut txt = "name my_filename.ext\ncrt Ax9lQnI\ncng NWzxOMo\nsz C4_A\npepr zeO1DgAAAADN47UOAAAAAM3jtQ4AAAAAzeO1DgAAAADN47UOAAAAAM3jtQ4AAAAAzeO1DgAAAADN47UOAAAAAA\npad 'Z\";&(-GRt~rTrp>_nb[2Lc*13!Bn)\nenc:\n".as_bytes();
        let (length, actual) = parse_private_header(&mut txt).unwrap();
        let expected = PrivateHeader::new(
            "my_filename.ext".to_owned(),
            None,
            Some(123_456_789_000),
            Some(987_654_321_000),
            None,
            1024_000,
            Salt::fixed_for_test(246_801_357),
            30,
        );
        assert_eq!(length, 185);
        assert_eq!(actual, expected);
    }
}
