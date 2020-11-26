use ::std::cmp::Ordering;
use ::std::collections::btree_set::Iter;
use ::std::collections::BTreeSet;
use ::std::fmt;
use ::std::fmt::Formatter;
use ::std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncOptionSet {
    options: BTreeSet<EncOption>,
}

impl From<Vec<EncOption>> for EncOptionSet {
    fn from(vector: Vec<EncOption>) -> Self {
        EncOptionSet::new(vector)
    }
}

impl EncOptionSet {
    pub fn new(options: Vec<EncOption>) -> Self {
        EncOptionSet {
            options: options.iter().cloned().collect(),
        }
    }

    pub fn empty() -> Self {
        EncOptionSet {
            options: BTreeSet::new(),
        }
    }

    #[cfg(test)]
    pub fn all_for_test() -> Self {
        vec![EncOption::Fast, EncOption::HideMeta, EncOption::PadSize].into()
    }

    pub fn len(&self) -> usize {
        self.options.len()
    }

    pub fn has(&self, option: EncOption) -> bool {
        self.options.contains(&option)
    }

    pub fn iter(&self) -> Iter<'_, EncOption> {
        self.options.iter()
    }
}

/// Encryption modifiers to use. Each should be used at most once.
#[derive(Debug, Clone, PartialEq, Eq, Ord, Hash)]
pub enum EncOption {
    Fast,
    HideMeta,
    PadSize,
}

impl EncOption {
    fn ordinal(&self) -> usize {
        match self {
            EncOption::Fast => 1,
            EncOption::HideMeta => 2,
            EncOption::PadSize => 3,
        }
    }
}

impl PartialOrd for EncOption {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.ordinal().cmp(&other.ordinal()))
    }
}

impl fmt::Display for EncOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Should NOT contain whitespace (whitespace is used as separator in the file header).
        write!(f, "{}", match self {
            EncOption::Fast => "fast",
            EncOption::HideMeta => "hide-meta",
            EncOption::PadSize => "pad-size",
        })
    }
}

impl FromStr for EncOption {
    type Err = ();

    fn from_str(txt: &str) -> Result<Self, Self::Err> {
        return Ok(match txt.to_ascii_lowercase().as_str() {
            "fast" => EncOption::Fast,
            "hide-meta" => EncOption::HideMeta,
            "pad-size" => EncOption::PadSize,
            _ => return Err(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod collection {
        use super::*;

        #[test]
        fn deduplicate() {
            let options = EncOptionSet::new(vec![
                EncOption::Fast,
                EncOption::HideMeta,
                EncOption::Fast,
            ]);
            assert_eq!(options.iter().count(), 2);
        }

        #[test]
        fn ordered() {
            let options = EncOptionSet::new(vec![
                EncOption::HideMeta,
                EncOption::Fast,
                EncOption::PadSize,
            ]);
            let mut options_iter = options.iter();
            assert_eq!(options_iter.next(), Some(&EncOption::Fast));
            assert_eq!(options_iter.next(), Some(&EncOption::HideMeta));
            assert_eq!(options_iter.next(), Some(&EncOption::PadSize));
        }

        #[test]
        fn has() {
            let options = EncOptionSet::new(vec![
                EncOption::HideMeta,
                EncOption::PadSize,
            ]);
            assert!(!options.has(&EncOption::Fast));
            assert!(options.has(&EncOption::HideMeta));
            assert!(options.has(&EncOption::PadSize));
        }
    }

    mod ordering {
        use super::*;

        #[test]
        fn sequence() {
            assert!(EncOption::Fast < EncOption::HideMeta);
        }
    }

    mod strings {
        use super::*;

        #[test]
        fn case_insentisive() {
            assert_eq!(EncOption::from_str("Fast"), Ok(EncOption::Fast));
            assert_eq!(EncOption::from_str("HIDE-META"), Ok(EncOption::HideMeta));
        }

        #[test]
        fn variant_fast() {
            let repr = "fast";
            assert_eq!(EncOption::Fast.to_string(), repr);
            assert_eq!(EncOption::from_str(repr), Ok(EncOption::Fast));
        }

        #[test]
        fn variant_hide_meta() {
            let repr = "hide-meta";
            assert_eq!(EncOption::HideMeta.to_string(), repr);
            assert_eq!(EncOption::from_str(repr), Ok(EncOption::HideMeta));
        }

        #[test]
        fn variant_hide_size() {
            let repr = "pad-size";
            assert_eq!(EncOption::PadSize.to_string(), repr);
            assert_eq!(EncOption::from_str(repr), Ok(EncOption::PadSize));
        }
    }
}
