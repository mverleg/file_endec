use crate::files::file_meta::FileInfo;
use crate::files::reading::open_reader;
use crate::header::{get_version_strategy, parse_public_header, PublicHeader, Strategy};
use crate::{FedResult, Verbosity};

#[derive(Debug)]
pub struct FileHeaderStrategy<'a> {
    pub file: &'a FileInfo<'a>,
    pub pub_header: PublicHeader,
    pub pub_header_len: usize,
    pub strategy: &'a Strategy,
}

impl<'a> FileHeaderStrategy<'a> {
    pub fn new(
        file: &'a FileInfo<'a>,
        header: PublicHeader,
        header_len: usize,
        verbosity: Verbosity,
    ) -> FedResult<Self> {
        let strategy = get_version_strategy(header.version(), header.options(), verbosity.debug())?;
        Ok(FileHeaderStrategy {
            file,
            pub_header: header,
            pub_header_len: header_len,
            strategy,
        })
    }
}

pub fn read_file_strategies<'a>(
    files: &'a [FileInfo],
    verbosity: Verbosity,
) -> FedResult<Vec<FileHeaderStrategy<'a>>> {
    files
        .iter()
        .map(|fi| (fi, open_reader(&fi, verbosity)))
        .map(|(fi, reader)| {
            (
                fi,
                reader.and_then(|mut r| parse_public_header(&mut r, verbosity.debug())),
            )
        })
        .map(|(fi, hdr_info)| {
            hdr_info.and_then(|(hdr_len, hdr)| FileHeaderStrategy::new(fi, hdr, hdr_len, verbosity))
        })
        .collect()
}

pub trait FileStrategy {
    fn file(&self) -> &FileInfo;
    fn strategy(&self) -> &Strategy;
}

impl<'a> FileStrategy for FileHeaderStrategy<'a> {
    fn file(&self) -> &FileInfo {
        self.file
    }

    fn strategy(&self) -> &Strategy {
        self.strategy
    }
}
