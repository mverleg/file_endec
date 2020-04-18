use crate::files::file_meta::FileInfo;
use crate::header::Strategy;
use crate::FedResult;

#[derive(Debug)]
pub struct FileStrategy<'a> {
    info: &'a FileInfo<'a>,
    strategy: &'a Strategy,
}

impl <'a> FileStrategy<'a> {
    pub fn new(
        info: &'a FileInfo<'a>,
        strategy: &'a Strategy,
    ) -> Self {
        FileStrategy {
            info,
            strategy,
        }
    }
}

pub fn read_file_strategies<'a>(files: &'a [FileInfo]) -> FedResult<Vec<FileStrategy<'a>>> {
    unimplemented!()
}
