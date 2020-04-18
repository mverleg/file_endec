use crate::files::file_meta::FileInfo;
use crate::header::{Strategy, Header, get_version_strategy};
use crate::{FedResult, Verbosity};

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

pub fn read_file_strategies<'a>(files: &'a [FileInfo]) -> FedResult<Vec<FileHeader<'a>>> {
    unimplemented!()  //TODO @mark:
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
