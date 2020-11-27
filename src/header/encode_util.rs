use ::std::error::Error;
use ::std::io::Write;

use crate::util::errors::add_err;
use crate::util::FedResult;

/// Use a space for separating key and value.
const KEY_VALUE_DELIMITER_CHAR: u8 = b' ';
/// Only \n newline is supported. While readable, easy of access on different operating
/// systems is not a goal, so use a short and consistent newline character.
const END_LINE_CHAR: u8 = b'\n';

fn wrap_err(res: Result<usize, impl Error>, verbose: bool) -> FedResult<()> {
    if let Err(err) = res {
        Err(add_err("failed to write encryption header", verbose, err))
    } else {
        Ok(())
    }
}

pub fn write_line(
    writer: &mut impl Write,
    prefix: &str,
    value: Option<&str>,
    verbose: bool,
) -> FedResult<()> {
    debug_assert!(!prefix.contains(KEY_VALUE_DELIMITER_CHAR));
    debug_assert!(!prefix.contains(END_LINE_CHAR));
    if let Some(val) = value {
        debug_assert!(!val.contains(END_LINE_CHAR));
    }
    wrap_err(writer.write(prefix.as_bytes()), verbose)?;
    if let Some(text) = value {
        wrap_err(writer.write(&[KEY_VALUE_DELIMITER_CHAR]), verbose)?;
        wrap_err(writer.write(text.as_bytes()), verbose)?;
    }
    wrap_err(writer.write(&[END_LINE_CHAR]), verbose)?;
    Ok(())
}
