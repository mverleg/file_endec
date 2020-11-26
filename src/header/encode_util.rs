use ::std::error::Error;
use ::std::io::Write;

use ::semver::Version;

use crate::EncOptionSet;
use crate::files::Checksum;
use crate::header::PublicHeader;
use crate::header::PUB_HEADER_CHECKSUM_MARKER;
use crate::header::PUB_HEADER_MARKER;
use crate::header::PUB_HEADER_SALT_MARKER;
use crate::header::PUB_HEADER_VERSION_MARKER;
use crate::header::public_header_type::{PUB_HEADER_META_DATA_MARKER, PUB_HEADER_OPTION_MARKER};
use crate::key::salt::Salt;
use crate::util::errors::add_err;
use crate::util::FedResult;
use crate::util::version::version_has_options_meta;

const DELIMITER_CHARS: [u8; 1] = [32,];

fn wrap_err(res: Result<usize, impl Error>, verbose: bool) -> FedResult<()> {
    if let Err(err) = res {
        Err(add_err("failed to write encryption header", verbose, err))
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
        wrap_err(writer.write(&DELIMITER_CHARS), verbose)?;
        wrap_err(writer.write(text.as_bytes()), verbose)?;
    }
    wrap_err(writer.write(b"\n"), verbose)?;
    Ok(())
}
