use ::std::error::Error;
use ::std::io::Write;

use crate::util::errors::add_err;
use crate::util::FedResult;

/// Use a space for separating key and value.
const KEY_VALUE_DELIMITER_CHARS: [u8; 1] = [b' ',];
/// Only \n newline is supported. While readable, easy of access on different operating
/// systems is not a goal, so use a short and consistent newline character.
const END_LINE_CHARS: [u8; 1] = [b'\n',];

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
    debug_assert!(prefix.as_bytes().windows(KEY_VALUE_DELIMITER_CHARS.len()).position(|window| window == KEY_VALUE_DELIMITER_CHARS).is_none());
    debug_assert!(prefix.as_bytes().windows(END_LINE_CHARS.len()).position(|window| window == END_LINE_CHARS).is_none());
    //TODO @mark: ^
    if let Some(val) = value {
        debug_assert!(val.as_bytes().windows(END_LINE_CHARS.len()).position(|window| window == END_LINE_CHARS).is_none());
    }
    wrap_err(writer.write(prefix.as_bytes()), verbose)?;
    if let Some(text) = value {
        wrap_err(writer.write(&KEY_VALUE_DELIMITER_CHARS), verbose)?;
        wrap_err(writer.write(text.as_bytes()), verbose)?;
    }
    wrap_err(writer.write(&END_LINE_CHARS), verbose)?;
    Ok(())
}
