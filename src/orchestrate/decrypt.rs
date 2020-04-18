use ::std::collections::HashMap;

use crate::config::typ::{EndecConfig, Extension};
use crate::config::DecryptConfig;
use crate::files::checksum::calculate_checksum;
use crate::files::compress::decompress_file;
use crate::files::file_meta::inspect_files;
use crate::files::read_headers::read_file_strategies;
use crate::files::write_output::write_output_file;
use crate::files::Checksum;
use crate::header::decode::skip_header;
use crate::key::key::StretchKey;
use crate::key::stretch::stretch_key;
use crate::key::Salt;
use crate::progress::Progress;
use crate::symmetric::decrypt::decrypt_file;

pub use crate::config::enc::EncryptConfig;
use crate::files::reading::{open_reader, read_file};
pub use crate::header::strategy::Verbosity;
pub use crate::key::{Key, KeySource};
use crate::progress::indicatif::IndicatifProgress;
pub use crate::util::FedResult;

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

pub fn decrypt(config: &DecryptConfig) -> FedResult<()> {
    if config.delete_input() {
        unimplemented!("deleting input not implemented"); //TODO @mark
    }
    let files_info = inspect_files(
        config.files(),
        config.verbosity(),
        config.overwrite(),
        Extension::Strip,
        config.output_dir(),
    )?;
    let files_strats = read_file_strategies(&files_info, config.verbosity())?;
    let mut progress = IndicatifProgress::new_dec_strategy(&files_strats, config.verbosity());
    let mut key_cache: HashMap<Salt, StretchKey> = HashMap::new();
    //TODO @mark: if I want to do time logging well, I need to scan headers to see how many salts
    let mut checksum_failure_count = 0;
    for file_strat in &files_strats {
        let mut reader = open_reader(&file_strat.file, config.verbosity())?;
        skip_header(&mut reader, config.verbosity().debug())?;
        let salt = file_strat.header.salt().clone();
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
        let data = read_file(
            &mut reader,
            &file_strat.file.path_str(),
            file_strat.file.size_kb,
            config.verbosity(),
            &mut || progress.start_read_for_file(&file_strat.file),
        )?;
        let revealed = decrypt_file(
            data,
            &stretched_key,
            &salt,
            &file_strat.strategy.symmetric_algorithms,
            &mut |alg| progress.start_sym_alg_for_file(alg, &file_strat.file)
        )?;
        let big = decompress_file(
            revealed,
            &file_strat.strategy.compression_algorithm,
            &mut |alg| progress.start_compress_alg_for_file(alg, &file_strat.file)
        )?;
        let actual_checksum = calculate_checksum(&big, &mut || progress.start_checksum_for_file(&file_strat.file));
        if !validate_checksum_matches(
            &actual_checksum,
            file_strat.header.checksum(),
            config.verbosity(),
            &file_strat.file.path_str(),
        ) {
            checksum_failure_count += 1;
        }
        write_output_file(config, &file_strat.file, &big, None, &mut || {
            progress.start_write_for_file(&file_strat.file)
        })?;
        if !config.quiet() {
            println!(
                "successfully decrypted '{}' to '{}' ({} kb)",
                &file_strat.file.path_str(),
                &file_strat.file.out_pth.to_string_lossy(),
                big.len() / 1024,
            );
        }
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
    Ok(())
}

/// The demo used in this blog post:
/// https://markv.nl/blog/symmetric-encryption-in-rust
#[cfg(test)]
mod tests {
    use ::std::fs;
    use ::std::fs::File;
    use ::std::io::Read;

    use ::lazy_static::lazy_static;
    use ::regex::Regex;
    use ::semver::Version;
    use tempfile::tempdir;

    use crate::config::DecryptConfig;
    use crate::decrypt;
    use crate::files::scan::get_enc_files_direct;
    use crate::files::scan::TEST_FILE_DIR;
    use crate::header::strategy::Verbosity;
    use crate::key::key::Key;

    lazy_static! {
        static ref COMPAT_KEY: Key = Key::new(" LP0y#shbogtwhGjM=*jFFZPmNd&qBO+ ");
        static ref COMPAT_FILE_RE: Regex = Regex::new(r"^original_v(\d+\.\d+\.\d+).png$").unwrap();
    }

    /// Open the files in 'test_files/' that were encrypted with previous versions,
    /// and make sure they can still be decrypted (and match the original).
    #[test]
    fn load_all_versions() {
        let enc_files: Vec<Version> = get_enc_files_direct(&*TEST_FILE_DIR)
            .unwrap()
            .iter()
            .map(|f| f.file_stem().unwrap().to_str().unwrap())
            .filter(|s| s.starts_with("original_"))
            .map(|n| COMPAT_FILE_RE.captures_iter(n).next().unwrap())
            .map(|v| Version::parse(&v[1]).unwrap())
            .collect();
        assert!(!enc_files.is_empty());
        let mut original_pth = TEST_FILE_DIR.clone();
        original_pth.push("original.png".to_owned());
        for enc_file in enc_files {
            let mut enc_pth = TEST_FILE_DIR.clone();
            enc_pth.push(format!("original_v{}.png.enc", enc_file));
            let mut dec_pth = TEST_FILE_DIR.clone();
            dec_pth.push(format!("original_v{}.png", enc_file));
            let conf = DecryptConfig::new(
                vec![enc_pth],
                COMPAT_KEY.clone(),
                Verbosity::Debug,
                true,
                false,
                None,
            );
            decrypt(&conf).unwrap();
            let mut original_data = vec![];
            File::open(&original_pth)
                .unwrap()
                .read_to_end(&mut original_data)
                .unwrap();
            let mut dec_data = vec![];
            File::open(&dec_pth)
                .unwrap()
                .read_to_end(&mut dec_data)
                .unwrap();
            assert_eq!(&original_data, &dec_data);
            fs::remove_file(&dec_pth).unwrap();
        }
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
            false,
            false,
            Some(out_pth.path().to_owned()),
        );
        let result = decrypt(&conf);
        assert!(&result.is_err());
        // dbg!(&result.unwrap_err());  //TODO @mark: TEMPORARY! REMOVE THIS!
        // panic!();
        assert!(&result.unwrap_err().contains("checksums did not match"));
    }

    //TODO @mark: check that invalid checksum will fail decryption
}
