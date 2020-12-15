use ::std::path::Path;
use ::std::path::PathBuf;
use ::std::sync::atomic::AtomicU32;
use ::std::sync::atomic::AtomicU64;
use ::std::sync::atomic::Ordering;

const LAST_INDEX: AtomicU64 = AtomicU64::new(1);

fn make_name(nr: u64, extension: &str) -> String {
    format!("{0:04}.{1:}", nr, extension)
}

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
    file.push(make_name(0, extension));
    loop {
        let nr = LAST_INDEX.fetch_add(1, Ordering::AcqRel);
        file.with_file_name(make_name(nr, extension));
        if !file.exists() {
            return file
        }
    }
}

#[cfg(test)]
mod tests {
    use ::std::fs;

    use ::tempfile::NamedTempFile;
    use ::tempfile::TempDir;

    use super::*;

    fn create_nr_files(directory: &Path, nrs: &[u64]) {
        for nr in nrs {
            let mut file = directory.to_path_buf();
            file.push(make_name(*nr, "enc"));
            fs::write(&file.as_path(), &vec![0]).unwrap();
        }
    }

    fn generate_available_pure_str_name(directory: &TempDir, extension: &str) -> String {
        generate_available_name(directory.path(), extension).file_name().unwrap().to_string_lossy().to_string()
    }

    #[test]
    fn first_available() {
        let dir = TempDir::new().unwrap();
        let name = generate_available_pure_str_name(&dir, "enc");
        assert_eq!(&name, "0001.enc");
    }

    #[test]
    fn at_end() {
        let dir = TempDir::new().unwrap();
        create_nr_files(dir.path(), &[1, 2, 3]);
        let name = generate_available_pure_str_name(&dir, "enc");
        assert_eq!(&name, "0004.enc");
    }

    #[test]
    fn gap() {
        let dir = TempDir::new().unwrap();
        create_nr_files(dir.path(), &[1, 2, 3, 5, 6]);
        let name = generate_available_pure_str_name(&dir, "enc");
        assert_eq!(&name, "0004.enc");
    }

    #[test]
    fn repeatedly() {
        let dir = TempDir::new().unwrap();
        create_nr_files(dir.path(), &[1, 2, 3, 5, 6, 8, 9,]);
        assert_eq!(&generate_available_pure_str_name(&dir, "enc"), "0004.enc");
        assert_eq!(&generate_available_pure_str_name(&dir, "enc"), "0007.enc");
        assert_eq!(&generate_available_pure_str_name(&dir, "enc"), "0010.enc");
    }

    #[test]
    fn extension() {
        let dir = TempDir::new().unwrap();
        create_nr_files(dir.path(), &[1, 2, 3, 5, 6, 8, 9,]);
        assert_eq!(&generate_available_pure_str_name(&dir, "enc"), "0004.enc");
        assert_eq!(&generate_available_pure_str_name(&dir, "enc"), "0007.enc");
        assert_eq!(&generate_available_pure_str_name(&dir, "new"), "0001.new");
        assert_eq!(&generate_available_pure_str_name(&dir, "new"), "0002.new");
    }
}