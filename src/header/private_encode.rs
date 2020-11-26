use ::std::io::Write;

use crate::header::encode_util::write_line;
use crate::header::private_header_type::{PRIV_HEADER_CHANGED, PRIV_HEADER_CREATED, PRIV_HEADER_DATA, PRIV_HEADER_FILENAME, PRIV_HEADER_PERMISSIONS, PRIV_HEADER_SIZE, PrivateHeader};
use crate::util::FedResult;

pub fn write_private_header(writer: &mut impl Write, header: &PrivateHeader, verbose: bool) -> FedResult<()> {
    write_line(writer, PRIV_HEADER_FILENAME, Some(header.filename()), verbose)?;
    write_line(writer, PRIV_HEADER_PERMISSIONS, Some(&format!("{:o}". header.permissions())), verbose)?;
    write_line(writer, PRIV_HEADER_CREATED, Some(&header.created_ns().as_base64()), verbose)?;
    write_line(writer, PRIV_HEADER_CHANGED, Some(&header.changed_ns().as_base64()), verbose)?;
    write_line(writer, PRIV_HEADER_SIZE, Some(&header.size().as_base64()), verbose)?;
    write_line(writer, PRIV_HEADER_DATA, None, verbose)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use ::std::str::from_utf8;

    use super::*;
    use super::write_public_header;

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
            "github.com/mverleg/file_endec\0\nv 1.1.0\nsalt AQAAAAAAAAABAAAAAAAAAAEAAAAAAAAAAQAAAAAAAAABAAAAAAAAAAEAAAAAAAAAAQAAAAAAAAABAAAAAAAAAA\ncheck xx_sha256 Ag\nmeta1+data:\n";
        assert_eq!(expected, from_utf8(&buf).unwrap());
    }
}
