use ::std::collections::HashMap;
use ::std::io::BufRead;

use crate::files::Checksum;
use crate::header::PUB_HEADER_CHECKSUM_MARKER;
use crate::header::PUB_HEADER_MARKER;
use crate::header::PUB_HEADER_PURE_DATA_MARKER;
use crate::header::PUB_HEADER_SALT_MARKER;
use crate::header::PUB_HEADER_VERSION_MARKER;
use crate::header::public_header_type::{PUB_HEADER_META_DATA_MARKER, PUB_HEADER_OPTION_MARKER};
use crate::header::PublicHeader;
use crate::key::salt::Salt;
use crate::util::errors::add_err;
use crate::util::FedResult;
use crate::util::option::{EncOption, EncOptionSet};
use crate::util::version::version_has_options_meta;

pub enum HeaderErr {
    NoStartMarker,
    NoEndMarker,
    // Either an system IO problem, or not valid utf8.
    ReadError,
}

fn read_line(reader: &mut dyn BufRead, line: &mut String) -> Result<(), HeaderErr> {
    line.clear();
    let res = reader.read_line(line);
    match res {
        Ok(sz) => if sz == 0 {
            return Err(HeaderErr::NoEndMarker)
        },
        Err(_) => return Err(HeaderErr::ReadError),
    }
    line.pop();
    Ok(())
}

pub fn read_header_keys(reader: &mut dyn BufRead, start: Option<&str>, ends: &[&str], verbose: bool) -> Result<HashMap<String, String>, HeaderErr> {
    assert!(!end.is_empty());
    let mut line = String::new();

    read_line(reader, &mut line)?;
    if let Some(start) = start {
        if line != start {
            return Err(HeaderErr::NoStartMarker)
        }
    }

    let mut map = HashMap::with_capacity(8);
    loop {
        read_line(reader, &mut line)?;
        // Currently only the end-markers end with a colon, but that may be temporary.
        for end in ends {
            if line == end {
                return Ok(map)
            }
        }
        debug_assert!(!line.ends_with(":"));

    }
}
