use ::std::path::Path;

use ::file_shred::shred_file;

use crate::files::file_meta::FileInfo;
use crate::{FedResult, Verbosity};

pub fn delete_existing_file_in_output_location(pth: &Path) -> FedResult<()> {
    assert!(pth.is_file());
    shred_file(&pth).map_err(|_| {
        "Failed to remove previously-existing file that exists in output location".to_string()
    })
}

pub fn delete_input_file(
    delete_input: bool,
    file: &FileInfo,
    start_progress: &mut impl FnMut(),
    verbosity: Verbosity,
) -> FedResult<()> {
    if delete_input {
        start_progress();
        shred_file(&file.in_path)?;
        if verbosity.debug() {
            println!("deleted {}", &file.file_name());
        }
    }
    Ok(())
}
