use crate::files::file_meta::FileInfo;
use crate::header::{Strategy, Header, get_version_strategy, parse_header};
use crate::{FedResult, Verbosity};
use crate::orchestrate::reading::open_reader;
use crate::files::reading::open_reader;

#[derive(Debug)]
pub struct FileHeader<'a> {
    pub file: &'a FileInfo<'a>,
    pub header: Header,
    pub strategy: &'a Strategy,
}

impl <'a> FileHeader<'a> {
    pub fn new(
        file: &'a FileInfo<'a>,
        header: Header,
        verbosity: &Verbosity,
    ) -> FedResult<Self> {
        let strategy = get_version_strategy(header.version(), verbosity.debug())?;
        Ok(FileHeader {
            file,
            header,
            strategy,
        })
    }
}

pub fn read_file_strategies<'a>(files: &'a [FileInfo], verbosity: Verbosity) -> FedResult<Vec<FileHeader<'a>>> {
    let strats = files.iter()
        .map(|fi| (fi, open_reader(&fi.file, verbosity)?))
        .map(|(fi, mut reader)| (fi, parse_header(&mut reader, verbosity.debug())))
        .collect();
    Ok(strats)
}

pub trait FileStrategy {
    fn file(&self) -> &FileInfo;
    fn strategy(&self) -> & Strategy;
}

impl <'a> FileStrategy for FileHeader<'a> {

    fn file(&self) -> &FileInfo {
        self.file
    }

    fn strategy(&self) -> &Strategy {
        self.strategy
    }
}
