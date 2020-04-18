use crate::files::file_meta::FileInfo;
use crate::header::{Strategy, Header, get_version_strategy};
use crate::{FedResult, Verbosity};

#[derive(Debug)]
pub struct FileStrategy<'a> {
    pub file: &'a FileInfo<'a>,
    pub header: Header,
    pub strategy: &'a Strategy,
}

impl <'a> FileStrategy<'a> {
    pub fn new(
        file: &'a FileInfo<'a>,
        header: Header,
        verbosity: &Verbosity,
    ) -> FedResult<Self> {
        let strategy = get_version_strategy(header.version(), verbosity.debug())?;
        Ok(FileStrategy {
            file,
            header,
            strategy,
        })
    }
}

pub fn read_file_strategies<'a>(files: &'a [FileInfo]) -> FedResult<Vec<FileStrategy<'a>>> {
    unimplemented!()  //TODO @mark:
}
