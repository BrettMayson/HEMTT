// mod ace;
pub mod cba;
// mod vanilla;

mod replace;
use std::{convert::TryFrom, fmt::Display, path::PathBuf};

pub use replace::{replace, Vars};

mod template;
pub use template::Template;

use crate::HEMTTError;

use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, Copy, EnumIter, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Templates {
    CBA,
}
impl Templates {
    pub fn validate<S: Into<String>>(template: S) -> Result<(), String> {
        if Self::try_from(template.into()).is_ok() {
            Ok(())
        } else {
            Err(Self::options())
        }
    }

    /// CLI - Valid options for CLI interfaces
    pub fn options() -> String {
        format!(
            "options are: {}",
            Self::iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
impl TryFrom<String> for Templates {
    type Error = ();
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "cba" => Ok(Self::CBA),
            _ => Err(()),
        }
    }
}
impl Display for Templates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::CBA => "cba",
        })
    }
}

pub fn init(template: Templates, path: PathBuf) -> Result<(), HEMTTError> {
    match template {
        Templates::CBA => cba::CBA::new(path).init(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate() {
        assert!(Templates::validate("cba").is_ok());
        assert!(Templates::validate("fake").is_err());
    }
}
