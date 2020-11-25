use ::criterion::criterion_group;
use ::criterion::criterion_main;

#[cfg(all(test, feature = "expose"))]
mod hash {
    use ::criterion::Benchmark;
    use ::criterion::black_box;
    use ::criterion::Criterion;

    use ::file_endec::get_current_version_strategy;
    use ::file_endec::hash_argon2i;
    use ::file_endec::hash_bcrypt;
    use ::file_endec::hash_sha256;
    use ::file_endec::Key;
    use ::file_endec::Salt;
    use ::file_endec::stretch_key;

    fn get_data() -> Vec<u8> {
        black_box(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16])
    }

    fn get_salt() -> Vec<u8> {
        black_box(vec![
            1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
            0, 1, 0,
        ])
    }

    pub fn scrypt_benchmark(c: &mut Criterion) {
        c.bench(
            "bcrypt",
            Benchmark::new("bcrypt", |b| {
                b.iter(|| hash_bcrypt(&get_data(), &get_salt()))
            })
            .sample_size(20),
        );
    }

    pub fn argon2id_benchmark(c: &mut Criterion) {
        c.bench(
            "argon2id",
            Benchmark::new("argon2id", |b| {
                b.iter(|| hash_argon2i(&get_data(), &get_salt()))
            })
            .sample_size(20),
        );
    }

    pub fn sha256_benchmark(c: &mut Criterion) {
        c.bench(
            "sha256_hash",
            Benchmark::new("sha256_hash", |b| {
                b.iter(|| hash_sha256(&get_data(), &get_salt()))
            })
            .sample_size(20),
        );
    }

    pub fn stretch_benchmark(c: &mut Criterion) {
        c.bench(
            "stretch",
            Benchmark::new("stretch", |b| {
                b.iter(|| {
                    let strat = get_current_version_strategy(&vec![].into(), false);
                    stretch_key(
                        &Key::new(&"MY secret p@ssw0rd"),
                        &Salt::fixed_for_test(123_456_789),
                        strat.stretch_count,
                        &strat.key_hash_algorithms,
                        &mut |_| (),
                    )
                })
            })
            .sample_size(10),
        );
    }
}

#[cfg(all(test, feature = "expose"))]
mod encrypt {
    use ::criterion::Benchmark;
    use ::criterion::black_box;
    use ::criterion::Criterion;

    use ::file_endec::decrypt_aes256;
    use ::file_endec::decrypt_twofish;
    use ::file_endec::encrypt_aes256;
    use ::file_endec::encrypt_twofish;
    use ::file_endec::generate_test_file_content_for_test;
    use ::file_endec::Salt;
    use ::file_endec::StretchKey;

    pub fn encrypt_aes256_benchmark(c: &mut Criterion) {
        c.bench(
            "enc_dec_aes256",
            Benchmark::new("enc_dec_aes256", |b| {
                let key = StretchKey::mock_stretch(b"1_s3cr3t_p@55w0rd!!");
                let salt = Salt::fixed_for_test(123_456_789_123_456_789);
                let input = generate_test_file_content_for_test(1_000_000);
                let input_start = input[..8].to_vec();
                let input_end = input[input.len() - 8..].to_vec();
                let expected_start = &[99, 98, 68, 40, 23, 127, 40, 229];
                let expected_end = &[18, 153, 235, 245, 136, 236, 90, 174];
                b.iter(|| {
                    let secret = encrypt_aes256(black_box(&input), &key, &salt);
                    assert_eq!(expected_start, &secret[..8]);
                    assert_eq!(expected_end, &secret[secret.len() - 8..]);
                    let back = decrypt_aes256(black_box(&secret), &key, &salt).unwrap();
                    assert_eq!(input_start, &back[..8]);
                    assert_eq!(input_end, &back[back.len() - 8..]);
                })
            })
            .sample_size(10),
        );
    }

    pub fn encrypt_twofish_benchmark(c: &mut Criterion) {
        c.bench(
            "enc_dec_twofish",
            Benchmark::new("enc_dec_twofish", |b| {
                let key = StretchKey::mock_stretch(b"1_s3cr3t_p@55w0rd!!");
                let salt = Salt::fixed_for_test(123_456_789_123_456_789);
                let input = generate_test_file_content_for_test(1_000_000);
                let input_start = input[..8].to_vec();
                let input_end = input[input.len() - 8..].to_vec();
                let expected_start = &[123, 234, 159, 158, 79, 48, 128, 175];
                let expected_end = &[126, 104, 211, 189, 140, 204, 62, 135];
                b.iter(|| {
                    let secret = encrypt_twofish(black_box(&input), &key, &salt);
                    assert_eq!(expected_start, &secret[..8]);
                    assert_eq!(expected_end, &secret[secret.len() - 8..]);
                    let back = decrypt_twofish(black_box(&secret), &key, &salt).unwrap();
                    assert_eq!(input_start, &back[..8]);
                    assert_eq!(input_end, &back[back.len() - 8..]);
                })
            })
            .sample_size(10),
        );
    }
}

#[cfg(all(test, feature = "expose"))]
mod back_and_forth {
    use ::std::fs::File;
    use ::std::io::Write;
    use ::std::path::PathBuf;

    use ::criterion::Benchmark;
    use ::criterion::black_box;
    use ::criterion::Criterion;
    use ::rand::RngCore;
    use ::tempfile::tempdir;

    use ::file_endec::encrypt;
    use ::file_endec::EncryptConfig;
    use ::file_endec::Key;
    use file_endec::{EncOptionSet, Verbosity, DecryptConfig, decrypt};

    fn create_test_file() -> PathBuf {
        //TODO @mark: reusable somewhere?
        let mut pth = tempdir().unwrap().into_path();
        pth.push("source.data");
        assert!(pth.exists());
        let mut data = [0; 1024 * 1024];
        rand::thread_rng().fill_bytes(&mut data);
        let mut file = File::create(&pth).unwrap();
        file.write_all(&data).unwrap();
        return pth
    }

    fn enc_dec_files_with_options(key: Key, test_file: PathBuf, options: EncOptionSet) {
        let conf = EncryptConfig::new(
            vec![test_file],
            key.clone(),
            options,
            Verbosity::Quiet,
            false,
            true,
            Some(pth.path().to_owned()),
            ".enc".to_string(),
            false,
        );
        let enc_files = encrypt(&conf).unwrap();

        let conf = DecryptConfig::new(
            enc_files,
            key,
            Verbosity::Quiet,
            false,
            false,
            None,
        );
        decrypt(&conf).unwrap();
    }

    pub fn v1_0(c: &mut Criterion) {
        // Note: this just uses current version without options, which is the same for v1.0.
        // There is currently no way to encrypt using older versions, so if defaults change,
        // this test might have to be scrapped.
        c.bench(
            "v1_0",
            Benchmark::new("v1_0", |b| {
                let key = Key::new("s$j2d@PBBajiX$1+&hMEEij@+XNrUR4u");;
                let version = get_current_version();
                let test_file = create_test_file();
                b.iter(|| enc_dec_files_with_options(key, test_file, EncOptionSet::empty()))
            })
            .sample_size(10),
        );
    }

    pub fn v1_1_fast(c: &mut Criterion) {
        c.bench(
            "v1_0",
            Benchmark::new("v1_0", |b| {
                let key = Key::new("TzBdMjzA8%++lSUdwxlak83jZg=veF4!");;
                let version = get_current_version();
                let test_file = create_test_file();
                b.iter(|| enc_dec_files_with_options(key, test_file, EncOptionSet::all_for_test()))
            })
            .sample_size(10),
        );
    }
}

#[cfg(not(feature = "expose"))]
pub fn need_expose_feature(_: &mut Criterion) {
    panic!("benchmarks require feature 'expose' to be enabled")
}

#[cfg(feature = "expose")]
criterion_group!(
    hash_bench,
    hash::scrypt_benchmark,
    hash::argon2id_benchmark,
    hash::sha256_benchmark,
    hash::stretch_benchmark,
);

#[cfg(feature = "expose")]
criterion_group!(
    encrypt_bench,
    encrypt::encrypt_aes256_benchmark,
    encrypt::encrypt_twofish_benchmark,
);

#[cfg(feature = "expose")]
criterion_group!(
    back_and_forth_bench,
    back_and_forth::v1_0,
    back_and_forth::v1_1_fast,
);

#[cfg(feature = "expose")]
criterion_main!(hash_bench, encrypt_bench, back_and_forth_bench);

#[cfg(not(feature = "expose"))]
criterion_group!(need_expose_feature_group, need_expose_feature);
#[cfg(not(feature = "expose"))]
criterion_main!(need_expose_feature_group);
