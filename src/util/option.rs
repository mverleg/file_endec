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

    pub fn len(&self) -> usize {
        self.options.len()
    }

    pub fn has(&self, option: &EncOption) -> bool {
        self.options.contains(option)
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
}

impl EncOption {
    fn ordinal(&self) -> usize {
        match self {
            EncOption::Fast => 1,
            EncOption::HideMeta => 2,
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
        write!(f, "{}", match self {
            EncOption::Fast => "fast",
            EncOption::HideMeta => "hide-meta",
        })
    }
}

impl FromStr for EncOption {
    type Err = ();

    fn from_str(txt: &str) -> Result<Self, Self::Err> {
        return Ok(match txt.to_ascii_lowercase().as_str() {
            "fast" => EncOption::Fast,
            "hide-meta" => EncOption::HideMeta,
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
            ]);
            let mut options_iter = options.iter();
            assert_eq!(options_iter.next(), Some(&EncOption::Fast));
            assert_eq!(options_iter.next(), Some(&EncOption::HideMeta));
        }

        #[test]
        fn has() {
            let options = EncOptionSet::new(vec![
                EncOption::HideMeta,
            ]);
            assert!(!options.has(&EncOption::Fast));
            assert!(options.has(&EncOption::HideMeta));
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
    }
}
