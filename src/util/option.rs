use ::std::fmt;
use ::std::fmt::Formatter;
use ::std::str::FromStr;
use std::collections::BTreeSet;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct EncOptions {
    options: BTreeSet<EncOption>,
}

impl EncOptions {
    pub fn new(options: Vec<EncOption>) -> Self {
        EncOptions {
            options: options.into(),
        }
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
