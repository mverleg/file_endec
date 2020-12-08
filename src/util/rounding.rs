use ::std::mem::size_of;

/// Round up to the nearest power of two, e.g. 16, 32, 64, 128...
pub fn round_up_to_power_of_two(value: u64) -> u64 {
    debug_assert!(value < 2u64.pow(63), "not implemented for numbers > 2**63");
    if value == 0 {
        return 0;
    }
    // The power of two, rounded up, is the number of bits needed to store the number.
    let maximum_bits = (size_of::<u64>() * 8) as u32;
    let bits_used = (value - 1).leading_zeros() as u32;
    let power = maximum_bits - bits_used;
    // The answer then is just 2^power.
    2u64.pow(power)
}

/// Number needed to reach the next-nearest power of two (as returned
/// by `round_up_to_power_of_two`).
pub fn remainder_to_power_of_two(value: u64) -> u64 {
    round_up_to_power_of_two(value) - value
}

#[cfg(test)]
mod tests {
    use super::*;

    mod round_up {
        use super::*;

        #[test]
        fn needs_rounding() {
            assert_eq!(round_up_to_power_of_two(7), 8);
            assert_eq!(round_up_to_power_of_two(13), 16);
            assert_eq!(round_up_to_power_of_two(1023), 1024);
            assert_eq!(round_up_to_power_of_two(1025), 2048);
            assert_eq!(round_up_to_power_of_two(2u64.pow(63) - 1), 2u64.pow(63));
        }

        #[test]
        fn already_rounded() {
            assert_eq!(round_up_to_power_of_two(1), 1);
            assert_eq!(round_up_to_power_of_two(8), 8);
            assert_eq!(round_up_to_power_of_two(4096), 4096);
        }

        #[test]
        fn zero() {
            assert_eq!(round_up_to_power_of_two(0), 0);
        }
    }

    mod remainder {
        use super::*;

        #[test]
        fn needs_rounding() {
            assert_eq!(remainder_to_power_of_two(7), 1);
            assert_eq!(remainder_to_power_of_two(13), 3);
            assert_eq!(remainder_to_power_of_two(1023), 1);
            assert_eq!(remainder_to_power_of_two(1025), 1023);
            assert_eq!(remainder_to_power_of_two(2u64.pow(63) - 1), 1);
        }

        #[test]
        fn already_rounded() {
            assert_eq!(remainder_to_power_of_two(1), 0);
            assert_eq!(remainder_to_power_of_two(8), 0);
            assert_eq!(remainder_to_power_of_two(4096), 0);
        }

        #[test]
        fn zero() {
            assert_eq!(remainder_to_power_of_two(0), 0);
        }
    }
}
