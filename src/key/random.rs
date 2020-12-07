use ::std::cell::RefCell;
use ::std::sync::Arc;
use ::std::sync::atomic::AtomicBool;
use ::std::sync::atomic::Ordering;
use ::std::thread::sleep;
use ::std::thread::spawn;
use ::std::time::Duration;
use ::std::time::SystemTime;

use ::rand::Rng;
use ::rand::RngCore;
use ::rand::rngs::OsRng;
use ::rand::rngs::StdRng;
use ::rand::SeedableRng;

thread_local! {
    // Note: implements CryptoRng  //TODO @mark: test <--
    //TODO @mark: is seeding secure enough?
    static RNG: RefCell<StdRng> = RefCell::new(StdRng::from_entropy());
}

/// Generate a secure random series of bytes, showing a
/// warning on stderr if it takes long.
pub fn generate_secure_random_timed(buffer: &mut [u8]) {

    let is_ready = Arc::new(AtomicBool::new(false));
    let has_warned = Arc::new(AtomicBool::new(false));
    let timer = SystemTime::now();

    // Spawn a thread just to log messages if things take long.
    let is_ready_monitor = is_ready.clone();
    let has_warned_monitor = has_warned.clone();
    spawn(move || {
        // Wait one second before warning.
        sleep(Duration::new(1, 0));
        if is_ready_monitor.load(Ordering::Acquire) {
            return;
        }
        has_warned_monitor.store(true, Ordering::Release);
        eprintln!("secure random number generation is taking long; perhaps there is not enough entropy available");
    });

    // This does the actual number generation.
    OsRng.fill_bytes(buffer);
    is_ready.store(true, Ordering::Release);

    // If the warning was shown, then also show that the situation is resolved now.
    if has_warned.load(Ordering::Acquire) {
        eprintln!("secure random number generation ready after {} ms", timer.elapsed().unwrap().as_millis());
    }
}

/// This is 'secure' in the sense that next or previous values aren't predictable; it is still pseudo-
/// random and not 'true' random.
pub fn generate_secure_pseudo_random_bytes(buffer: &mut Vec<u8>, length: usize) {
    buffer.clear();
    RNG.with(|rng| {
        let mut rng = rng.borrow_mut();
        for _ in 0..length {
            buffer.push(rng.gen())
            //TODO: would this be faster in batches?
        }
    })
}

/// Secure psuedo-random like `generate_secure_pseudo_random_bytes`, but characters are in most
/// of the non-whitespace, printable ascii range.
pub fn generate_secure_pseudo_random_printable(buffer: &mut String, length: usize) {
    buffer.clear();
    RNG.with(|rng| {
        let mut rng = rng.borrow_mut();
        for _ in 0..length {
            buffer.push(rng.gen_range(33, 127) as u8 as char)
            //TODO: would this be faster in batches?
        }
    })
}

#[cfg(test)]
mod tests {
    use ::rand::CryptoRng;

    use super::*;

    fn test_is_secure() -> impl CryptoRng {
        // This fails at compile time if not cryptographic.
        RNG.with(|rng| rng.borrow().clone())
    }

    const N: usize = 100;

    #[test]
    fn bytes() {
        let mut data = Vec::with_capacity(N);
        generate_secure_pseudo_random_bytes(&mut data, N);
        assert_eq!(data.len(), N);
        let total: u64 = data.into_iter().map(|v| v as u64).sum();
        assert!(total > 0, "all random bytes were 0; this has probability < 1e-240");
    }

    #[test]
    fn printable() {
        let mut data = String::with_capacity(N);
        generate_secure_pseudo_random_printable(&mut data, N);
        assert_eq!(data.len(), N);
        let total: u64 = data.as_bytes().iter().map(|v| *v as u64).sum();
        assert!(total > 0, "all random characters were 0; this has probability < 1e-240");
    }
}
