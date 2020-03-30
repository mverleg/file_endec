use ::criterion::criterion_group;
use ::criterion::criterion_main;
use criterion::Criterion;

#[cfg(all(test, feature = "expose"))]
mod hash {
    use ::criterion::Benchmark;
    use ::criterion::black_box;
    use ::criterion::Criterion;

    use ::file_endec::header::strategy::get_current_version_strategy;
    use ::file_endec::key::{Key, Salt};
    use ::file_endec::key::hash::hash_argon2i;
    use ::file_endec::key::hash::hash_bcrypt;
    use ::file_endec::key::hash::hash_sha256;
    use ::file_endec::key::stretch::stretch_key;

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
                    let strat = get_current_version_strategy(true);
                    stretch_key(
                        &Key::new(&"MY secret p@ssw0rd"),
                        &Salt::fixed_for_test(123_456_789),
                        strat.stretch_count,
                        &strat.key_hash_algorithms,
                    )
                })
            })
            .sample_size(5),
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

#[cfg(not(feature = "expose"))]
pub fn need_expose_feature(c: &mut Criterion) {
    panic!("benchmarks require feature 'expose' to be enabled")
}

//TODO @mark: fully encrypt and decrypt large file

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
criterion_main!(hash_bench,
    hash_bench,
    encrypt_bench,
);

#[cfg(not(feature = "expose"))]
criterion_main!(hash_bench, need_expose_feature);