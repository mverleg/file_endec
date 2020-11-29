use ::std::sync::Arc;
use ::std::sync::atomic::AtomicBool;
use ::std::sync::atomic::Ordering;
use ::std::thread::sleep;
use ::std::thread::spawn;
use ::std::time::Duration;
use ::std::time::SystemTime;

use ::rand::RngCore;
use ::rand::rngs::OsRng;

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
        sleep(Duration::new(100, 0));
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
