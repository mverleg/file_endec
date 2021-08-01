use ::std::borrow::BorrowMut;
use ::std::cell::RefCell;
use ::std::io::Write;

use crate::{EncOption, EncOptionSet};
use crate::header::encode_util::write_line;
use crate::header::private_header_type::{PRIV_HEADER_ACCESSED, PRIV_HEADER_CREATED, PRIV_HEADER_DATA, PRIV_HEADER_FILENAME, PRIV_HEADER_MODIFIED, PRIV_HEADER_PADDING, PRIV_HEADER_PEPPER, PRIV_HEADER_PERMISSIONS, PRIV_HEADER_DATA_SIZE_CHECK, PrivateHeader};
use crate::key::random::generate_secure_pseudo_random_printable;
use crate::util::base::u128_to_small_str;
use crate::util::base::u64_to_small_str;
use crate::util::base::u8s_to_base64str;
use crate::util::FedResult;

thread_local! {
    static BUFFER: RefCell<String> = RefCell::new(String::with_capacity(256));
}

fn write_padding(length: u16, write: impl FnOnce(&str) -> FedResult<()>) -> FedResult<()> {
    // Use a per-thread shared buffer to prevent allocations.
    BUFFER.with(|buf| {
        let mut padding = buf.borrow_mut();
        generate_secure_pseudo_random_printable(padding.borrow_mut(), length as usize);
        write(&padding)
    })
}

pub fn write_private_header(writer: &mut impl Write, header: &PrivateHeader, options: &EncOptionSet, verbose: bool) -> FedResult<()> {
    write_line(writer, PRIV_HEADER_FILENAME, Some(header.filename()), verbose)?;
    if options.has(EncOption::HideMeta) {
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
    //if options.has(EncOption::PadSize) {  //TODO @mark: keep it required? even if not used?
    write_line(writer, PRIV_HEADER_DATA_SIZE_CHECK, Some(&u64_to_small_str(header.data_size())), verbose)?;
    write_line(writer, PRIV_HEADER_PEPPER, Some(&u8s_to_base64str(&header.pepper().salt)), verbose)?;
    write_padding(header.padding_len(), |pad| write_line(writer, PRIV_HEADER_PADDING, Some(pad), verbose))?;
    write_line(writer, PRIV_HEADER_DATA, None, verbose)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use ::std::str::from_utf8;

    use crate::key::Salt;

    use super::*;

    #[test]
    fn write_vanilla() {
        let header = PrivateHeader::new(
            "my_filename.ext".to_owned(),
            None,
            None,
            None,
            None,
            1024_000,
            Salt::fixed_for_test(010_101_010),
            0,
        );
        let mut buf: Vec<u8> = Vec::new();
        write_private_header(&mut buf, &header, &EncOptionSet::empty(), true).unwrap();
        let expected =
            "name my_filename.ext\nsz C4_A\npepr EiGaAAAAAAASIZoAAAAAABIhmgAAAAAAEiGaAAAAAAASIZoAAAAAABIhmgAAAAAAEiGaAAAAAAASIZoAAAAAAA\npad \nenc:\n";
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
            Salt::fixed_for_test(246_801_357),
            10,
        );
        let mut buf: Vec<u8> = Vec::new();
        write_private_header(&mut buf, &header, &EncOptionSet::all_for_test(), true).unwrap();
        let txt = from_utf8(&buf).unwrap();
        let expected_prefix =
            "name my_filename.ext\nperm 754\ncrt Ax9lQnI\ncng NWzxOMo\nacs NiToP-_\nsz C4_A\npepr zeO1DgAAAADN47UOAAAAAM3jtQ4AAAAAzeO1DgAAAADN47UOAAAAAM3jtQ4AAAAAzeO1DgAAAADN47UOAAAAAA\npad ";
        let expected_postfix = "\nenc:\n";
        assert!(txt.starts_with(expected_prefix));
        assert!(txt.ends_with(expected_postfix));
        assert_eq!(txt.len(), expected_prefix.len() + expected_postfix.len() + 10);
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
            Salt::fixed_for_test(246_801_357),
            30,
        );
        let mut buf: Vec<u8> = Vec::new();
        write_private_header(&mut buf, &header, &EncOptionSet::all_for_test(), true).unwrap();
        let txt = from_utf8(&buf).unwrap();
        let expected_prefix =
            "name my_filename.ext\ncrt Ax9lQnI\ncng NWzxOMo\nsz C4_A\npepr zeO1DgAAAADN47UOAAAAAM3jtQ4AAAAAzeO1DgAAAADN47UOAAAAAM3jtQ4AAAAAzeO1DgAAAADN47UOAAAAAA\npad ";
        let expected_postfix = "\nenc:\n";
        assert!(txt.starts_with(expected_prefix));
        assert!(txt.ends_with(expected_postfix));
        assert_eq!(txt.len(), expected_prefix.len() + expected_postfix.len() + 30);
    }
}
