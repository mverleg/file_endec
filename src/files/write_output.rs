use ::std::fs::File;
use ::std::io::Write;

use ::file_shred::shred_file;

use crate::config::typ::EndecConfig;
use crate::files::file_meta::FileInfo;
use crate::header::Header;
use crate::header::write_header;
use crate::util::errors::wrap_io;
use crate::util::FedResult;

pub fn write_output_file(
    config: &impl EndecConfig,
    file: &FileInfo,
    data: &[u8],
    header: Option<&Header>,
    start_progress: &mut impl FnMut(),
) -> FedResult<()> {
    start_progress();
    if file.out_pth.exists() {
        if config.overwrite() {
            assert!(file.out_pth.is_file());
            shred_file(&file.out_pth).map_err(|_| {
                "Failed to remove previously-existing file that exists in output location"
                    .to_string()
            })?;
        } else {
            return Err(format!(
                "While encrypting, a file appeared in previously empty \
                        output location '{}'",
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
        write_header(&mut out_file, &header, config.debug())?;
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
        println!("encrypted {}", &file.out_pth.to_string_lossy());
    }
    Ok(())
}
