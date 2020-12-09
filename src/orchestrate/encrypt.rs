use ::std::path::PathBuf;

use crate::config::enc::EncryptConfig;
use crate::config::typ::{EndecConfig, Extension};
use crate::files::Checksum;
use crate::files::checksum::calculate_checksum;
use crate::files::compress::compress_file;
use crate::files::delete::delete_input_file;
use crate::files::file_meta::{FileInfo, inspect_files};
use crate::files::reading::{open_reader, read_file};
use crate::files::write_output::write_output_file;
use crate::header::private_encode::write_private_header;
use crate::header::private_header_type::PrivateHeader;
use crate::header::{PublicHeader, Strategy};
use crate::header::strategy::get_current_version_strategy;
use crate::header::strategy::Verbosity;
use crate::key::key::StretchKey;
use crate::key::Salt;
use crate::key::stretch::stretch_key;
use crate::progress::indicatif::IndicatifProgress;
use crate::progress::log::LogProgress;
use crate::progress::Progress;
use crate::progress::silent::SilentProgress;
use crate::symmetric::encrypt::encrypt_file;
use crate::util::errors::FedResult;
use crate::util::option::EncOption;
use crate::util::version::get_current_version;
use crate::key::random::generate_secure_pseudo_random_bytes;
use crate::util::rounding::remainder_to_power_of_two;

//TODO @mark: I need to add some random number of bytes to private header, because the attacker knows the size of the cyphertext, so they can deduce private header information

fn encrypt_private_header(salt: &Salt, key: &StretchKey, pepper: &Salt, file: &FileInfo, strategy: &Strategy, config: &EncryptConfig, data_encrypted_size: u64, start_progress: &mut impl FnMut()) -> FedResult<(Vec<u8>, Checksum)> {
    start_progress();
    // This padding length has expectation value 256, which is probably enough to obfuscate most filename lengths.
    let padding_len = (pepper.salt[0] as u16) + (pepper.salt[1] as u16);
    let priv_header = PrivateHeader::new(
        file.file_name(),
        file.permissions,
        file.created_ns,
        file.changed_ns,
        file.accessed_ns,
        data_encrypted_size,
        //TODO @mark: should this be pepper or salt?
        pepper.clone(),
        padding_len,
    );
    let mut data = Vec::with_capacity(2048);
    write_private_header(
        &mut data,
        &priv_header,
        config.options(),
        config.verbosity().debug()
    )?;
    let checksum = calculate_checksum(&data, &mut || {});
    let secret = encrypt_file(
        data,
        key,
        salt,
        &strategy.symmetric_algorithms,
        &mut |_| {},
    );
    Ok((secret, checksum))
}

/// Encrypt one or more files and return the new paths.
pub fn encrypt(config: &EncryptConfig) -> FedResult<Vec<PathBuf>> {
    //TODO @mark: break this up into more functions?
    if config.options().has(EncOption::HideMeta) { eprintln!("metadata hiding not yet implemented"); }  //TODO @mark: TEMPORARY! REMOVE THIS!
    if config.options().has(EncOption::PadSize) { eprintln!("size hiding not yet implemented"); }  //TODO @mark: TEMPORARY! REMOVE THIS!
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
    // Public and private salt are different from eachother, but the same for all files.
    let salt = Salt::generate_random()?;
    let pepper = Salt::generate_random()?;
    let stretched_key = stretch_key(
        config.raw_key(),
        &salt,
        strategy.stretch_count,
        &strategy.key_hash_algorithms,
        &mut |alg| progress.start_stretch_alg(&alg, None),
    );
    let mut file_padding = Vec::with_capacity(4096);
    let mut out_pths = vec![];
    for file in &files_info {
        let mut reader = open_reader(&file, config.verbosity())?;
        let mut data = Vec::with_capacity(file.size_b as usize + 10_240);
        read_file(
            &mut data,
            &mut reader,
            &file.path_str(),
            file.size_kb(),
            config.verbosity(),
            &mut || progress.start_read_for_file(&file),
        )?;
        // Do not include the private header in the checksum (by skipping it).
        let data_checksum = calculate_checksum(&data, &mut || progress.start_checksum_for_file(&file));

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

        //TODO @mark: move data checksum into private header for v1.1
        let (priv_header_data, priv_header_checksum) = encrypt_private_header(
            &salt, &stretched_key, &pepper, file, &strategy, config, secret.len() as u64,
            &mut || progress.start_private_header_for_file(&file))?;
        let priv_header_len = priv_header_data.len() as u64;

        let padding_len = remainder_to_power_of_two((priv_header_data.len() + secret.len()) as u64) as usize;
        generate_secure_pseudo_random_bytes(&mut file_padding, padding_len);
        let pub_header = PublicHeader::new(version.clone(), salt.clone(), data_checksum, config.options().clone(), (priv_header_len, priv_header_checksum));
        if !config.dry_run() {
            write_output_file(config, &file.out_pth, &[&priv_header_data, &secret, &file_padding], Some(&pub_header), &mut || {
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
