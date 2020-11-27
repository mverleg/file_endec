use ::std::collections::HashMap;
use ::std::io::BufRead;

#[derive(Debug)]
pub enum HeaderErr {
    NoStartMarker,
    NoEndMarker,
    HeaderSyntax(String),
    // Either an system IO problem, or not valid utf8.
    ReadError,
}

fn read_line(reader: &mut dyn BufRead, line: &mut String, index: &mut usize) -> Result<(), HeaderErr> {
    line.clear();
    let res = reader.read_line(line);
    match res {
        Ok(sz) => if sz == 0 {
            return Err(HeaderErr::NoEndMarker)
        },
        Err(err) => {
            dbg!(err);  //TODO @mark: TEMPORARY! REMOVE THIS!
            return Err(HeaderErr::ReadError)
        },
    }
    *index = *index + line.len();
    line.pop();
    eprintln!("line: {}", &line);  //TODO @mark: TEMPORARY! REMOVE THIS!
    Ok(())
}

/// Read a header of this format:
///   START (optional)
///   key1 value1
///   key2 value2
///   ...
///   END: (one of several options)
/// Also return the index right after the end marker.
pub fn read_header_keys(reader: &mut dyn BufRead, start: Option<&str>, ends: &[&str]) -> Result<(usize, HashMap<String, String>), HeaderErr> {
    assert!(!ends.is_empty());
    let mut line = String::new();
    let mut index = 0;

    read_line(reader, &mut line, &mut index)?;
    if let Some(start) = start {
        if line != start {
            return Err(HeaderErr::NoStartMarker)
        }
        read_line(reader, &mut line, &mut index)?;
    }

    let mut map = HashMap::with_capacity(8);
    loop {
        if line.trim().is_empty() {
            read_line(reader, &mut line, &mut index)?;
            continue;
        }

        for end in ends {
            if &line == end {
                return Ok((index, map))
            }
        }
        // Currently, only the end-markers end with a colon, but that may be temporary.
        debug_assert!(!line.ends_with(':'));

        let mut parts = line.splitn(2, ' ');
        let key = parts.next().unwrap().to_owned();
        let value = match parts.next() {
            Some(val) => val.to_owned(),
            None => return Err(HeaderErr::HeaderSyntax(line)),
        };
        map.insert(key, value);

        read_line(reader, &mut line, &mut index)?;
    }
}

//TODO @mark: test (copy from public_decode?)
pub fn skip_header<R: BufRead>(reader: &mut R, ends: &[&str]) -> Result<(), HeaderErr> {
    let mut line = String::new();
    loop {
        read_line(reader, &mut line, &mut 0)?;
        for end in ends {
            if &line == end {
                return Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ::std::io::BufReader;

    use super::*;

    mod read_keys {
        use super::*;

        #[test]
        fn empty() {
            let input = "hello\0\nworld:\nignore this";
            let mut reader = BufReader::new(input.as_bytes());
            let (index, map) = read_header_keys(&mut reader, Some("hello\0"), &vec!["world:"]).unwrap();
            assert_eq!(index, 17);
            assert!(map.is_empty());
        }

        #[test]
        fn no_start_empty() {
            let input = "end:\nignore this";
            let mut reader = BufReader::new(input.as_bytes());
            let (index, map) = read_header_keys(&mut reader, None, &vec!["end:"]).unwrap();
            assert_eq!(index, 4);
            assert!(map.is_empty());
        }

        #[test]
        fn no_start_single_end() {
            let input = "key value\nend:\nignore this";
            let mut reader = BufReader::new(input.as_bytes());
            let (index, map) = read_header_keys(&mut reader, None, &vec!["end:"]).unwrap();
            assert_eq!(index, 15);
            assert!(!map.is_empty());
            assert_eq!(map.get("key").map(|v| v.as_str()), Some("value"));
            assert_eq!(map.get("other"), None);
        }

        #[test]
        fn start_double_end() {
            let input = "start\0\nkey value\nletters alpha beta gamma\nend2:\nignore this";
            let mut reader = BufReader::new(input.as_bytes());
            let (index, map) = read_header_keys(&mut reader, Some("start\0"), &vec!["end1:", "end2:"]).unwrap();
            assert_eq!(index, 51);
            assert!(!map.is_empty());
            assert_eq!(map.get("key").map(|v| v.as_str()), Some("value"));
            assert_eq!(map.get("letters").map(|v| v.as_str()), Some("alpha beta gamma"));
            assert_eq!(map.get("other"), None);
        }

        #[test]
        fn skip_empty_lines() {
            let input = "start\0\n  \nkey value\n\nletters alpha beta gamma\n\nend2:\nignore this";
            let mut reader = BufReader::new(input.as_bytes());
            let (index, map) = read_header_keys(&mut reader, Some("start\0"), &vec!["end1:", "end2:"]).unwrap();
            assert_eq!(index, 59);
            assert!(!map.is_empty());
            assert_eq!(map.get("key").map(|v| v.as_str()), Some("value"));
            assert_eq!(map.get("letters").map(|v| v.as_str()), Some("alpha beta gamma"));
            assert_eq!(map.get("other"), None);
        }

        //TODO @mark: error situations
    }
}