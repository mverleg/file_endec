use ::std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

/// Encryption modifiers to use. Each should be used at most once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncOption {
    Fast,
    HideMeta,
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
