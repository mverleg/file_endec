use crate::files::file_meta::FileInfo;
use crate::files::reading::open_reader;
use crate::header::{get_version_strategy, parse_public_header, PublicHeader, Strategy};
use crate::{FedResult, Verbosity};

#[derive(Debug)]
pub struct FileHeader<'a> {
    pub file: &'a FileInfo<'a>,
    pub header: PublicHeader,
    pub strategy: &'a Strategy,
}

impl<'a> FileHeader<'a> {
    pub fn new(file: &'a FileInfo<'a>, header: PublicHeader, verbosity: Verbosity) -> FedResult<Self> {
        let strategy = get_version_strategy(header.version(), header.options(), verbosity.debug())?;
        Ok(FileHeader {
            file,
            header,
            strategy,
        })
    }
}

pub fn read_file_strategies<'a>(
    files: &'a [FileInfo],
    verbosity: Verbosity,
) -> FedResult<Vec<FileHeader<'a>>> {
    files
        .iter()
        .map(|fi| (fi, open_reader(&fi, verbosity)))
        .map(|(fi, reader)| {
            (
                fi,
                reader.and_then(|mut r| parse_public_header(&mut r, verbosity.debug())),
            )
        })
        .map(|(fi, header)| header.and_then(|h| FileHeader::new(fi, h, verbosity)))
        .collect()
}

pub trait FileStrategy {
    fn file(&self) -> &FileInfo;
    fn strategy(&self) -> &Strategy;
}

impl<'a> FileStrategy for FileHeader<'a> {
    fn file(&self) -> &FileInfo {
        self.file
    }

    fn strategy(&self) -> &Strategy {
        self.strategy
    }
}
