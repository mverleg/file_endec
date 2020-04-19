#![cfg(any(test, feature = "expose"))]

#[cfg(test)]
use ::tempfile::{TempDir, NamedTempFile};
use ::std::fs;

mod tests {
    #[allow(unused_imports)]
    use super::generate_test_file_content_for_test;

    #[test]
    fn generate() {
        let data = generate_test_file_content_for_test(15_001);
        assert_eq!(15_001, data.len());
        assert!(data.contains(&0));
        assert!(data.contains(&127));
        assert!(data.contains(&255));
    }
}

pub fn generate_test_file_content_for_test(len: usize) -> Vec<u8> {
    let mut data = vec![0u8; len];
    let mut a: u32 = 1;
    let mut b: u32 = 1;
    #[allow(clippy::needless_range_loop)]
    for i in 0..len {
        let c = (a + b) % 256;
        data[i] = c as u8;
        a = b;
        b = c;
    }
    data
}

#[cfg(test)]
pub fn write_test_file(len: usize) -> NamedTempFile {
    let dir = TempDir::new().unwrap();
    let pth = NamedTempFile::new_in(dir.path()).unwrap();
    let big = generate_test_file_content_for_test(len);
    fs::write(&pth, big).unwrap();
    pth
}
