use ::std::path::PathBuf;

use crate::{EncOption, EncryptConfig, FedResult, Verbosity};
use crate::config::typ::{EndecConfig, Extension};
use crate::files::checksum::calculate_checksum;
use crate::files::compress::compress_file;
use crate::files::delete::delete_input_file;
use crate::files::file_meta::inspect_files;
use crate::files::reading::{open_reader, read_file};
use crate::files::write_output::write_output_file;
use crate::header::Header;
use crate::header::strategy::get_current_version_strategy;
use crate::key::Salt;
use crate::key::stretch::stretch_key;
use crate::progress::indicatif::IndicatifProgress;
use crate::progress::log::LogProgress;
use crate::progress::Progress;
use crate::progress::silent::SilentProgress;
use crate::symmetric::encrypt::encrypt_file;
use crate::util::version::get_current_version;

/// Encrypt one or more files and return the new paths.
pub fn encrypt(config: &EncryptConfig) -> FedResult<Vec<PathBuf>> {
    assert!(!config.options().has(&EncOption::HideMeta), "metadata hiding not yet implemented");  //TODO @mark: TEMPORARY! REMOVE THIS!
    assert!(!config.options().has(&EncOption::PadSize), "size hiding not yet implemented");  //TODO @mark: TEMPORARY! REMOVE THIS!
    let version = get_current_version();
    let strategy = get_current_version_strategy(config.options(), config.debug());
    let files_info = inspect_files(
        config.files(),
        config.verbosity(),
        config.overwrite(),
        Extension::Add(config.output_extension()),
        config.output_dir(),
    )?;
    let mut progress: Box<dyn Progress> = match config.verbosity() {
        Verbosity::Quiet => Box::new(SilentProgress::new()),
        Verbosity::Normal => Box::new(IndicatifProgress::new_enc_strategy(
            &strategy,
            &files_info,
            config.delete_input(),
            config.verbosity(),
        )),
        Verbosity::Debug => Box::new(LogProgress::new()),
    };
    let salt = Salt::generate_random()?;
    let stretched_key = stretch_key(
        config.raw_key(),
        &salt,
        strategy.stretch_count,
        &strategy.key_hash_algorithms,
        &mut |alg| progress.start_stretch_alg(&alg, None),
    );
    let mut out_pths = vec![];
    for file in &files_info {
        let mut reader = open_reader(&file, config.verbosity())?;
        let data = read_file(
            &mut reader,
            &file.path_str(),
            file.size_kb,
            config.verbosity(),
            &mut || progress.start_read_for_file(&file),
        )?;
        let checksum = calculate_checksum(&data, &mut || progress.start_checksum_for_file(&file));
        let small = compress_file(data, &strategy.compression_algorithm, &mut |alg| {
            progress.start_compress_alg_for_file(&alg, &file)
        })?;
        let secret = encrypt_file(
            small,
            &stretched_key,
            &salt,
            &strategy.symmetric_algorithms,
            &mut |alg| progress.start_sym_alg_for_file(&alg, &file),
        );
        let header = Header::new(version.clone(), salt.clone(), checksum, config.options().clone())?;
        if !config.dry_run() {
            write_output_file(config, &file, &secret, Some(&header), &mut || {
                progress.start_write_for_file(&file)
            })?;
            //TODO @mark: test that file is removed?
            delete_input_file(
                config.delete_input(),
                file,
                &mut || progress.start_shred_input_for_file(&file),
                config.verbosity(),
            )?;
        } else if !config.quiet() {
            progress.start_write_for_file(&file);
            println!(
                "successfully encrypted '{}' ({} kb); not saving to '{}' because of dry-run",
                file.path_str(),
                secret.len() / 1024,
                &file.out_pth.to_string_lossy(),
            );
        }
        out_pths.push(file.out_pth.clone());
    }
    progress.finish();
    if !config.quiet() {
        println!("encrypted {} files", files_info.len());
    }
    Ok(out_pths)
}

/// The demo used in this blog post:
/// https://markv.nl/blog/symmetric-encryption-in-rust
#[cfg(test)]
mod tests {
    use ::std::fs;

    use ::lazy_static::lazy_static;
    use ::regex::Regex;
    use tempfile::tempdir;

    use crate::config::EncryptConfig;
    use crate::encrypt;
    use crate::files::scan::TEST_FILE_DIR;
    use crate::header::strategy::Verbosity;
    use crate::key::key::Key;
    use crate::util::option::EncOptionSet;
    use crate::util::version::get_current_version;

    lazy_static! {
        static ref COMPAT_KEY: Key = Key::new(" LP0y#shbogtwhGjM=*jFFZPmNd&qBO+ ");
        static ref COMPAT_FILE_RE: Regex = Regex::new(r"^original_v(\d+\.\d+\.\d+).png$").unwrap();
    }

    struct Variation {
        postfix: String,
        options: EncOptionSet,
    }

    fn variations() -> Vec<Variation> {
        // Options Fast and HideMeta are supported from version 1.1.0
        // Prefix should be unique, and either empty in regex ^_\w+$
        vec![
            Variation {
                postfix: "".to_owned(),
                options: vec![].into(),
            },
            Variation {
                postfix: "_fast_hide".to_owned(),
                options: EncOptionSet::all_for_test(),
            },
        ]
    }

    #[test]
    fn store_current_version() {
        let dir = tempdir().unwrap();
        let version = get_current_version();
        let in_pth = {
            let mut p = TEST_FILE_DIR.clone();
            p.push("original.png");
            p
        };
        assert!(in_pth.exists());
        for variation in variations() {
            let conf = EncryptConfig::new(
                vec![in_pth.clone()],
                COMPAT_KEY.clone(),
                //TODO @mark: try different options
                variation.options,
                Verbosity::Debug,
                true,
                false,
                Some(dir.path().to_owned()),
                ".enc".to_string(),
                false,
            );
            let tmp_pth = {
                let mut p = dir.path().to_owned();
                p.push("original.png.enc");
                p
            };
            encrypt(&conf).unwrap();
            assert!(tmp_pth.is_file(), "encrypted file was not created");
            let store_pth = {
                let mut p = TEST_FILE_DIR.clone();
                p.push(format!("original_v{}{}.png.enc", version, &variation.postfix));
                p
            };
            if !store_pth.exists() {
                println!("storing file for new version {} as part of backward compatibility test files:\n{} -> {}",
                         version, &tmp_pth.to_string_lossy(), &store_pth.to_string_lossy());
                fs::copy(&tmp_pth, &store_pth).unwrap();
            }
            // Remove the temporary file (as a courtesy, not critical).
            println!(
                "removing temporary file {} for version {}",
                &tmp_pth.to_string_lossy(),
                version
            );
        }
    }
}
