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

fn read_line(reader: &mut dyn BufRead, line: &mut String) -> Result<(), HeaderErr> {
    line.clear();
    let res = reader.read_line(line);
    match res {
        Ok(sz) => if sz == 0 {
            return Err(HeaderErr::NoEndMarker)
        },
        Err(err) => return Err(HeaderErr::ReadError),
    }
    line.pop();
    Ok(())
}

pub fn read_header_keys(reader: &mut dyn BufRead, start: Option<&str>, ends: &[&str]) -> Result<HashMap<String, String>, HeaderErr> {
    assert!(!ends.is_empty());
    let mut line = String::new();

    read_line(reader, &mut line)?;
    if let Some(start) = start {
        if line != start {
            return Err(HeaderErr::NoStartMarker)
        }
    }

    let mut map = HashMap::with_capacity(8);
    loop {
        read_line(reader, &mut line)?;
        for end in ends {
            if &line == end {
                return Ok(map)
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
    }
}

//TODO @mark: test (copy from public_decode?)
pub fn skip_header<R: BufRead>(reader: &mut R, ends: &[&str]) -> Result<(), HeaderErr> {
    let mut line = String::new();
    loop {
        read_line(reader, &mut line)?;
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
            let map = read_header_keys(&mut reader, Some("hello\0"), &vec!["world:"]).unwrap();
            assert!(map.is_empty());
        }

        #[test]
        fn no_start_single_end() {
            let input = "key value\nend:\nignore this";
            let mut reader = BufReader::new(input.as_bytes());
            let map = read_header_keys(&mut reader, None, &vec!["end:"]).unwrap();
            assert!(!map.is_empty());
            assert_eq!(map.get("key").map(|v| v.as_str()), Some("value"));
            assert_eq!(map.get("other"), None);
        }

        #[test]
        fn start_double_end() {
            let input = "start\0\nkey value\nletters alpha beta gamma\nend2:\nignore this";
            let mut reader = BufReader::new(input.as_bytes());
            let map = read_header_keys(&mut reader, None, &vec!["end1:", "end2:"]).unwrap();
            assert!(!map.is_empty());
            assert_eq!(map.get("key").map(|v| v.as_str()), Some("value"));
            assert_eq!(map.get("letters").map(|v| v.as_str()), Some("alpha beta gamma"));
            assert_eq!(map.get("other"), None);
        }

        //TODO @mark: error situations
    }
}