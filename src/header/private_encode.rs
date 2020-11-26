use ::std::io::Write;

use crate::{EncOption, EncOptionSet};
use crate::header::encode_util::write_line;
use crate::header::private_header_type::{PRIV_HEADER_ACCESSED, PRIV_HEADER_MODIFIED, PRIV_HEADER_CREATED, PRIV_HEADER_DATA, PRIV_HEADER_FILENAME, PRIV_HEADER_PERMISSIONS, PRIV_HEADER_SIZE, PrivateHeader};
use crate::util::base::u128_to_small_str;
use crate::util::base::u64_to_small_str;
use crate::util::FedResult;

pub fn write_private_header(writer: &mut impl Write, header: &PrivateHeader, options: &EncOptionSet, verbose: bool) -> FedResult<()> {
    if options.has(EncOption::HideMeta) {
        write_line(writer, PRIV_HEADER_FILENAME, Some(header.filename()), verbose)?;
        if let Some(perms) = header.permissions() {
            write_line(writer, PRIV_HEADER_PERMISSIONS, Some(&format!("{:o}", perms)), verbose)?;
        }
        if let Some(time_ns) = header.created_ns() {
            write_line(writer, PRIV_HEADER_CREATED, Some(&u128_to_small_str(time_ns)), verbose)?;
        }
        if let Some(time_ns) = header.changed_ns() {
            write_line(writer, PRIV_HEADER_MODIFIED, Some(&u128_to_small_str(time_ns)), verbose)?;
        }
        if let Some(time_ns) = header.accessed_ns() {
            write_line(writer, PRIV_HEADER_ACCESSED, Some(&u128_to_small_str(time_ns)), verbose)?;
        }
    }
    if options.has(EncOption::PadSize) {
        write_line(writer, PRIV_HEADER_SIZE, Some(&u64_to_small_str(header.size())), verbose)?;
    }
    write_line(writer, PRIV_HEADER_DATA, None, verbose)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use ::std::str::from_utf8;

    use super::*;

    #[test]
    fn write_vanilla() {
        let header = PrivateHeader::new(
            "my_filename.ext".to_owned(),
            Some(0o754),
            Some(123_456_789_000),
            Some(987_654_321_000),
            Some(999_999_999_999),
            1024_000,
        );
        let mut buf: Vec<u8> = Vec::new();
        write_private_header(&mut buf, &header, &EncOptionSet::empty(), true).unwrap();
        let expected =
            "enc:\n";
        assert_eq!(expected, from_utf8(&buf).unwrap());
    }

    #[test]
    fn write_hide_meta_size() {
        let header = PrivateHeader::new(
            "my_filename.ext".to_owned(),
            Some(0o754),
            Some(123_456_789_000),
            Some(987_654_321_000),
            Some(999_999_999_999),
            1024_000,
        );
        let mut buf: Vec<u8> = Vec::new();
        write_private_header(&mut buf, &header, &EncOptionSet::all_for_test(), true).unwrap();
        let expected =
            "name my_filename.ext\nperm 754\ncrt Ax9lQnI\ncng NWzxOMo\nacs NiToP-_\nsz C4_A\nenc:\n";
        assert_eq!(expected, from_utf8(&buf).unwrap());
    }

    #[test]
    fn write_hide_unsupported() {
        let header = PrivateHeader::new(
            "my_filename.ext".to_owned(),
            None,
            Some(123_456_789_000),
            Some(987_654_321_000),
            None,
            1024_000,
        );
        let mut buf: Vec<u8> = Vec::new();
        write_private_header(&mut buf, &header, &EncOptionSet::all_for_test(), true).unwrap();
        let expected =
            "name my_filename.ext\ncrt Ax9lQnI\ncng NWzxOMo\nsz C4_A\nenc:\n";
        assert_eq!(expected, from_utf8(&buf).unwrap());
    }
}
