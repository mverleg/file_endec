use ::file_shred::shred_file;

use crate::{FedResult, Verbosity};
use crate::files::file_meta::FileInfo;

pub fn delete_existing_file_in_output_location(file: &FileInfo) -> FedResult<()> {
    assert!(file.out_pth.is_file());
    shred_file(&file.out_pth).map_err(|_| {
        "Failed to remove previously-existing file that exists in output location"
            .to_string()
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
