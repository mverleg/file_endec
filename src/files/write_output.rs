use ::std::fs::File;
use ::std::io::Write;

use crate::config::typ::EndecConfig;
use crate::files::delete::delete_existing_file_in_output_location;
use crate::files::file_meta::FileInfo;
use crate::header::write_public_header;
use crate::header::PublicHeader;
use crate::util::errors::wrap_io;
use crate::util::FedResult;

pub fn write_output_file(
    config: &impl EndecConfig,
    file: &FileInfo,
    data: &[u8],
    header: Option<&PublicHeader>,
    start_progress: &mut impl FnMut(),
) -> FedResult<()> {
    start_progress();
    if file.out_pth.exists() {
        if config.overwrite() {
            delete_existing_file_in_output_location(&file)?;
        } else {
            return Err(format!(
                "While encrypting, a file appeared in previously empty output location '{}'",
                &file.out_pth.to_string_lossy()
            ));
        }
    }
    let mut out_file = wrap_io(
        || {
            format!(
                "Could not create output file for '{}'",
                &file.out_pth.to_string_lossy()
            )
        },
        File::create(&file.out_pth),
    )?;
    if let Some(header) = header {
        write_public_header(&mut out_file, &header, config.debug())?;
    }
    wrap_io(
        || {
            format!(
                "Failed to write encrypted output data for '{}'",
                &file.out_pth.to_string_lossy()
            )
        },
        out_file.write_all(&data),
    )?;
    if config.debug() {
        println!("encrypted {}", &file.file_name());
    }
    Ok(())
}
