use ::std::io::Write;

use crate::header::encode_util::write_line;
use crate::header::private_header_type::{PRIV_HEADER_CHANGED, PRIV_HEADER_CREATED, PRIV_HEADER_DATA, PRIV_HEADER_FILENAME, PRIV_HEADER_PERMISSIONS, PRIV_HEADER_SIZE, PrivateHeader};
use crate::util::base::u64_to_base64str;
use crate::util::base::u128_to_base64str;
use crate::util::FedResult;
use crate::{EncOptionSet, EncOption};

pub fn write_private_header(writer: &mut impl Write, header: &PrivateHeader, options: &EncOptionSet, verbose: bool) -> FedResult<()> {
    if options.has(EncOption::HideMeta) {
        write_line(writer, PRIV_HEADER_FILENAME, Some(header.filename()), verbose)?;
        write_line(writer, PRIV_HEADER_PERMISSIONS, Some(&format!("{:o}", header.permissions())), verbose)?;
        write_line(writer, PRIV_HEADER_CREATED, Some(&u128_to_base64str(header.created_ns())), verbose)?;
        write_line(writer, PRIV_HEADER_CHANGED, Some(&u128_to_base64str(header.changed_ns())), verbose)?;
    }
    if options.has(EncOption::PadSize) {
        write_line(writer, PRIV_HEADER_SIZE, Some(&u64_to_base64str(header.size())), verbose)?;
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
            0o754,
            123_456_789_000,
            987_654_321_000,
            1024_000,
        );
        let mut buf: Vec<u8> = Vec::new();
        write_private_header(&mut buf, &header, true).unwrap();
        let expected =
            "name my_filename.ext\nperm 754\ncrt CBqZvhwAAAAAAAAAAAAAAA\ncng aPPI9OUAAAAAAAAAAAAAAA\nsz AKAPAAAAAAA\nenc:\n";
        assert_eq!(expected, from_utf8(&buf).unwrap());
    }
}
