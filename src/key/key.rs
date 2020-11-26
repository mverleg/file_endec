use ::secstr::SecUtf8;
use ::secstr::SecVec;
use ::zxcvbn::zxcvbn;
use ::zxcvbn::Entropy;

#[cfg(any(test, feature = "expose"))]
use crate::key::hash::fastish_hash;

#[derive(Debug, Clone)]
pub struct Key {
    pub key_data: SecUtf8,
    pub strength: Entropy,
}

impl Key {
    pub fn new(key_data: &str) -> Self {
        let strength = zxcvbn(key_data, &[]).unwrap();
        Key {
            key_data: SecUtf8::from(key_data),
            strength,
        }
    }

    pub fn is_strong(&self) -> bool {
        self.strength.score() >= 3
    }

    pub fn time_to_crack(&self) -> String {
        format!(
            "{}",
            self.strength
                .crack_times()
                .offline_slow_hashing_1e4_per_second()
        )
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.key_data == other.key_data
    }
}

impl Eq for Key {}

#[derive(Debug, Clone)]
pub struct StretchKey {
    key_data: SecVec<u8>,
}

impl StretchKey {
    pub fn new(key_data: &[u8]) -> Self {
        debug_assert!(key_data.len() >= 32);
        StretchKey {
            key_data: SecVec::<u8>::from(key_data),
        }
    }

    #[cfg(any(test, feature = "expose"))]
    pub fn mock_stretch(key_data: &[u8]) -> Self {
        StretchKey::new(&fastish_hash(key_data))
    }

    pub fn len(&self) -> usize {
        self.key_data.unsecure().len()
    }

    pub fn unsecure_slice(&self, upto: usize) -> &[u8] {
        &self.key_data.unsecure()[..upto]
    }
}
