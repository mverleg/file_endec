use ::std::fs::File;
use ::std::io::Write;
use ::std::path::Path;

use crate::config::typ::EndecConfig;
use crate::files::delete::delete_existing_file_in_output_location;
use crate::header::PublicHeader;
use crate::header::write_public_header;
use crate::util::errors::wrap_io;
use crate::util::FedResult;

pub fn write_output_file(
    config: &impl EndecConfig,
    out_pth: &Path,
    datas: &[&[u8]],
    header: Option<&PublicHeader>,
    start_progress: &mut impl FnMut(),
) -> FedResult<()> {
    debug_assert!(datas.len() >= 1);
    start_progress();
    if out_pth.exists() {
        if config.overwrite() {
            delete_existing_file_in_output_location(&out_pth)?;
        } else {
            return Err(format!(
                "While encrypting, a file appeared in previously empty output location '{}'",
                &out_pth.to_string_lossy()
            ));
        }
    }
    let mut out_file = wrap_io(
        || {
            format!(
                "Could not create output file for '{}'",
                &out_pth.to_string_lossy()
            )
        },
        File::create(&out_pth),
    )?;
    if let Some(header) = header {
        write_public_header(&mut out_file, header, config.debug())?;
    }
    wrap_io(
        || {
            format!(
                "Failed to write encrypted output data for '{}'",
                &out_pth.to_string_lossy()
            )
        },
        datas.iter()
            .map(|data| out_file.write_all(data))
            .collect::<Result<Vec<_>, _>>(),
    )?;
    if config.debug() {
        println!("encrypted {}", out_pth
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string());
    }
    Ok(())
}
