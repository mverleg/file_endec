use ::std::collections::HashMap;
use ::std::io::Seek;
use ::std::io::SeekFrom;
use ::std::path::PathBuf;

use crate::config::typ::{EndecConfig, Extension};
use crate::config::DecryptConfig;
use crate::files::checksum::calculate_checksum;
use crate::files::compress::decompress_file;
use crate::files::delete::delete_input_file;
use crate::files::file_meta::inspect_files;
use crate::files::read_headers::read_file_strategies;
use crate::files::reading::{open_reader, read_file};
use crate::files::write_output::write_output_file;
use crate::files::Checksum;
use crate::header::private_decode::parse_private_header;
use crate::key::key::StretchKey;
use crate::key::stretch::stretch_key;
use crate::key::Salt;
use crate::progress::indicatif::IndicatifProgress;
use crate::progress::log::LogProgress;
use crate::progress::silent::SilentProgress;
use crate::progress::Progress;
use crate::symmetric::decrypt::decrypt_file;
use crate::util::version::version_has_options_meta;
use crate::{FedResult, Verbosity};

pub fn validate_checksum_matches(
    actual_checksum: &Checksum,
    expected_checksum: &Checksum,
    verbosity: Verbosity,
    file_name: &str,
) -> bool {
    if actual_checksum == expected_checksum {
        return true;
    }
    if verbosity.quiet() {
        return false;
    }
    eprintln!(
        "warning: checksum for '{}' did not match! the decrypted file may contain garbage{}",
        file_name,
        if verbosity.debug() {
            format!(
                " (expected {}, actually {})",
                expected_checksum, actual_checksum
            )
        } else {
            "".to_owned()
        }
    );
    false
}

/// Decrypt one or more files and return the new paths.
pub fn decrypt(config: &DecryptConfig) -> FedResult<Vec<PathBuf>> {
    //TODO @mark: break this up into more functions?
    let files_info = inspect_files(
        config.files(),
        config.verbosity(),
        config.overwrite(),
        Extension::Strip,
        config.output_dir(),
    )?;
    let files_strats = read_file_strategies(&files_info, config.verbosity())?;
    let mut progress: Box<dyn Progress> = match config.verbosity() {
        Verbosity::Quiet => Box::new(SilentProgress::new()),
        Verbosity::Normal => Box::new(IndicatifProgress::new_dec_strategy(
            &files_strats,
            config.delete_input(),
            config.verbosity(),
        )),
        Verbosity::Debug => Box::new(LogProgress::new()),
    };
    let mut key_cache: HashMap<Salt, StretchKey> = HashMap::new();
    let mut checksum_failure_count = 0;
    let mut out_pths = vec![];
    for file_strat in &files_strats {
        let mut reader = open_reader(&file_strat.file, config.verbosity())?;
        reader
            .seek(SeekFrom::Start(file_strat.pub_header_len as u64))
            .unwrap();
        let salt = file_strat.pub_header.salt().clone();
        let stretched_key = if let Some(sk) = key_cache.get(&salt) {
            sk.clone()
        } else {
            let sk = stretch_key(
                config.raw_key(),
                &salt,
                file_strat.strategy.stretch_count,
                &file_strat.strategy.key_hash_algorithms,
                &mut |alg| progress.start_stretch_alg(&alg, Some(&file_strat.file)),
            );
            key_cache.insert(salt.clone(), sk.clone());
            sk
        };
        let mut data = Vec::with_capacity(file_strat.file.size_b as usize);
        read_file(
            &mut data,
            &mut reader,
            &file_strat.file.path_str(),
            file_strat.file.size_kb(),
            config.verbosity(),
            &mut || progress.start_read_for_file(&file_strat.file),
        )?;
        let priv_header_len = file_strat
            .pub_header
            .private_header()
            .as_ref()
            .map(|hdr| hdr.0 as usize)
            .unwrap_or(0);
        let priv_header = if version_has_options_meta(&file_strat.pub_header.version()) {
            dbg!(&data[..priv_header_len]); //TODO @mark: TEMPORARY! REMOVE THIS!
            let index_header = parse_private_header(&mut &data[..priv_header_len])?;
            Some(index_header.1)
        } else {
            None
        };
        let revealed = decrypt_file(
            data,
            priv_header_len,
            &stretched_key,
            &salt,
            &file_strat.strategy.symmetric_algorithms,
            &mut |alg| progress.start_sym_alg_for_file(alg, &file_strat.file),
        )?;
        let big = decompress_file(
            revealed,
            &file_strat.strategy.compression_algorithm,
            &mut |alg| progress.start_compress_alg_for_file(alg, &file_strat.file),
        )?;
        let actual_checksum = calculate_checksum(&big, &mut || {
            progress.start_checksum_for_file(&file_strat.file)
        });
        if !validate_checksum_matches(
            &actual_checksum,
            file_strat.pub_header.checksum(),
            config.verbosity(),
            &file_strat.file.path_str(),
        ) {
            checksum_failure_count += 1;
        }
        write_output_file(config, &file_strat.file, &big, None, &mut || {
            progress.start_write_for_file(&file_strat.file)
        })?;
        if config.delete_input() {
            delete_input_file(
                config.delete_input(),
                &file_strat.file,
                &mut || progress.start_shred_input_for_file(&file_strat.file),
                config.verbosity(),
            )?;
        }
        if !config.quiet() {
            println!(
                "successfully decrypted '{}' to '{}' ({} kb)",
                &file_strat.file.path_str(),
                &file_strat.file.out_pth.to_string_lossy(),
                big.len() / 1024,
            );
        }
        out_pths.push(file_strat.file.out_pth.clone());
    }
    progress.finish();
    if !config.quiet() {
        println!("decrypted {} files", files_strats.len());
    }
    if checksum_failure_count > 0 {
        return Err(format!(
            "there were {} files whose checksums did not match; they \
        likely do not contain real data",
            checksum_failure_count
        ));
    }
    Ok(out_pths)
}

/// The demo used in this blog post:
/// https://markv.nl/blog/symmetric-encryption-in-rust
#[cfg(test)]
mod tests {
    use ::lazy_static::lazy_static;
    use ::tempfile::tempdir;

    use crate::config::typ::{InputAction, OnFileExist};
    use crate::config::DecryptConfig;
    use crate::decrypt;
    use crate::files::scan::TEST_FILE_DIR;
    use crate::header::strategy::Verbosity;
    use crate::key::key::Key;

    lazy_static! {
        static ref COMPAT_KEY: Key = Key::new(" LP0y#shbogtwhGjM=*jFFZPmNd&qBO+ ");
    }

    #[test]
    fn fail_invalid_checksum() {
        let mut enc_pth = TEST_FILE_DIR.clone();
        enc_pth.push("invalid_checksum.txt.enc".to_owned());
        let out_pth = tempdir().unwrap();
        let conf = DecryptConfig::new(
            vec![enc_pth],
            COMPAT_KEY.clone(),
            Verbosity::Normal,
            OnFileExist::Fail,
            InputAction::Keep,
            Some(out_pth.path().to_owned()),
        );
        let result = decrypt(&conf);
        assert!(&result.is_err());
        assert!(&result.unwrap_err().contains("checksums did not match"));
    }
}
