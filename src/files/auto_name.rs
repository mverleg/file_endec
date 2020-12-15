use ::std::path::Path;
use ::std::path::PathBuf;
use ::std::sync::atomic::AtomicU32;
use ::std::sync::atomic::AtomicU64;
use ::std::sync::atomic::Ordering;

const LAST_INDEX: AtomicU64 = AtomicU64::new(1);

/// Generate the next available numeric name, starting from `0001.enc`.
///
/// Fills gaps, i.e. if `0001.enc`, `0002.enc` and `0004.enc` exist,
/// the next is `0003.enc` (before `0005.enc`). So no binary search.
///
/// Subsequent calls will start from where the previous one stopped,
/// effectively assuming that no files are deleted between calls.
///
/// Continues to find names after `9999.enc` (`10000.enc`), but not that
/// having so many files would involve 10.000 disk operations.
pub fn generate_available_name(directory: &Path, extension: &str) -> PathBuf {
    assert!(directory.is_dir());
    let mut file = directory.to_path_buf();
    file.push(format!("0000.{}", extension));
    loop {
        let nr = LAST_INDEX.fetch_add(1, Ordering::AcqRel);
        file.with_file_name(format!("{0:04}.{1:}", nr, extension));
        if !file.exists() {
            return file
        }
    }
}
