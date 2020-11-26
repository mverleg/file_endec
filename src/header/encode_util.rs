use ::std::error::Error;
use ::std::io::Write;

use crate::util::errors::add_err;
use crate::util::FedResult;

const DELIMITER_CHARS: [u8; 1] = [32,];

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
    wrap_err(writer.write(prefix.as_bytes()), verbose)?;
    if let Some(text) = value {
        wrap_err(writer.write(&DELIMITER_CHARS), verbose)?;
        wrap_err(writer.write(text.as_bytes()), verbose)?;
    }
    wrap_err(writer.write(b"\n"), verbose)?;
    Ok(())
}
