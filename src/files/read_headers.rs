use crate::files::file_meta::FileInfo;
use crate::header::Strategy;
use crate::FedResult;

#[derive(Debug)]
pub struct FileStrategy<'a> {
    info: FileInfo<'a>,
    strategy: Strategy,
}

impl FileStrategy {
    pub fn new(
        info: FileInfo,
        strategy: Strategy,
    ) -> Self {
        FileStrategy {
            info,
            strategy,
        }
    }
}

pub fn read_file_strategies(files: &[FileInfo]) -> FedResult<Vec<FileStrategy>> {
    unimplemented!()
}
