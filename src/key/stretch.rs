use crate::header::KeyHashAlg;
use crate::key::hash::hash;
use crate::key::key::StretchKey;
use crate::key::Key;
use crate::key::Salt;

pub fn stretch_key(
    raw_key: &Key,
    salt: &Salt,
    stretch_count: u64,
    key_hash_algorithms: &[KeyHashAlg],
    start_progress: &mut impl FnMut(&KeyHashAlg),
) -> StretchKey {
    assert!(!key_hash_algorithms.is_empty());
    let salt_bytes = salt.salt;
    let mut data = raw_key.key_data.clone().unsecure().as_bytes().to_owned();
    for key_hash_alg in key_hash_algorithms {
        start_progress(&key_hash_alg);
        data = hash(&data, &salt_bytes, key_hash_alg);
        for i in 0..stretch_count {
            data.extend(&i.to_le_bytes());
            data = hash(&data, &salt_bytes, key_hash_alg);
        }
    }
    StretchKey::new(&data)
}

#[cfg(test)]
mod tests {
    #[cfg(not(debug_assertions))]
    use crate::header::strategy::get_current_version_strategy;

    #[cfg(not(debug_assertions))]
    use crate::util::option::EncOptionSet;

    #[cfg(not(debug_assertions))]
    use super::*;

    #[cfg(not(debug_assertions))]
    #[test]
    fn stratch_test_password_v1_0() {
        let strat = get_current_version_strategy(&EncOptionSet::empty(), true);
        let stretched = stretch_key(
            &Key::new(&"MY secret p@ssw0rd"),
            &Salt::fixed_for_test(123_456_789),
            strat.stretch_count,
            &strat.key_hash_algorithms,
            &mut |_| (),
        );
        assert_eq!(
            stretched.unsecure_slice(16),
            StretchKey::new(&[
                54, 114, 70, 167, 155, 254, 12, 193, 207, 39, 32, 139, 34, 157, 121, 67
            ])
            .unsecure_slice(16)
        );
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn stratch_test_password_v1_1_fast() {
        let strat = get_current_version_strategy(&EncOptionSet::all_for_test(), true);
        let stretched = stretch_key(
            &Key::new(&"MY secret p@ssw0rd"),
            &Salt::fixed_for_test(123_456_789),
            strat.stretch_count,
            &strat.key_hash_algorithms,
            &mut |_| (),
        );
        assert_eq!(
            stretched.unsecure_slice(16),
            StretchKey::new(&[
                112, 209, 30, 127, 161, 177, 105, 199, 59, 230, 70, 150, 183, 12, 238, 220
            ])
            .unsecure_slice(16)
        );
    }

    #[cfg(debug_assertions)]
    #[test]
    #[ignore]
    fn stratch_test_password() {
        panic!("Test skipped in debug mode, because it is really slow");
    }
}
