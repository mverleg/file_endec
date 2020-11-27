use ::std::fs::File;
use ::std::io::{BufReader, Read};

use crate::files::file_meta::FileInfo;
use crate::header::strategy::Verbosity;
use crate::header::PUB_HEADER_MARKER;
use crate::util::errors::wrap_io;
use crate::util::FedResult;

pub fn open_reader(file: &FileInfo, verbosity: Verbosity) -> FedResult<BufReader<File>> {
    match File::open(file.in_path) {
        Ok(file) => Ok(BufReader::new(file)),
        Err(err) => Err(if verbosity.debug() {
            format!("could not open input file {}: {}", file.path_str(), err)
        } else {
            format!("could not open input file {}", file.path_str())
        }),
    }
}

pub fn read_file(
    data: &mut Vec<u8>,
    reader: &mut BufReader<File>,
    path_str: &str,
    size_kb: u64,
    verbosity: Verbosity,
    start_progress: &mut impl FnMut(),
) -> FedResult<()> {
    start_progress();
    if verbosity.debug() {
        println!("reading {}", path_str);
    }
    if !verbosity.quiet() && size_kb > 1024 * 1024 {
        eprintln!(
            "warning: reading {} Mb file '{}' into RAM",
            size_kb / 1024,
            path_str
        );
    }
    wrap_io(|| "could not read input file", reader.read_to_end(data))?;

    if !verbosity.quiet() && data.starts_with(PUB_HEADER_MARKER.as_bytes()) {
        //TODO @mark: this is called during decrypt, how does this not happen too often?
        eprintln!("warning: file '{}' seems to already be encrypted", path_str);
    }
    Ok(())
}
