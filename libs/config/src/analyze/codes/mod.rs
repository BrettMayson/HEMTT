use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codes {
    InvalidValue,
    InvalidValueMacro,
    DuplicateProperty,
    MissingSemicolon,
    UnexpectedArray,
    ExpectedArray,
}

impl Codes {
    pub const fn ident(self) -> &'static str {
        match self {
            Self::InvalidValue => "CE1",
            Self::InvalidValueMacro => "CE2",
            Self::DuplicateProperty => "CE3",
            Self::MissingSemicolon => "CE4",
            Self::UnexpectedArray => "CE5",
            Self::ExpectedArray => "CE6",
        }
    }

    pub fn message(self) -> String {
        match self {
            Self::InvalidValue => "property's value could not be parsed.".to_string(),
            Self::InvalidValueMacro => "macro's result could not be parsed".to_string(),
            Self::DuplicateProperty => "property was defined more than once".to_string(),
            Self::MissingSemicolon => "property is missing a semicolon".to_string(),
            Self::UnexpectedArray => "property was not expected to be an array".to_string(),
            Self::ExpectedArray => "property was expected to be an array".to_string(),
        }
    }

    pub fn label_message(self) -> String {
        match self {
            Self::InvalidValue => "invalid value".to_string(),
            Self::InvalidValueMacro => "invalid macro result".to_string(),
            Self::DuplicateProperty => "duplicate property".to_string(),
            Self::MissingSemicolon => "missing semicolon".to_string(),
            Self::UnexpectedArray => "unexpected array".to_string(),
            Self::ExpectedArray => "expected array".to_string(),
        }
    }

    pub fn help(self) -> Option<String> {
        match self {
            Self::InvalidValue => Some(
                "use quotes `\"` around the value, or a QUOTE macro if it contains #define values"
                    .to_string(),
            ),
            Self::InvalidValueMacro => {
                Some("perhaps this macro has a `Q_` variant or you need `QUOTE(..)`".to_string())
            }
            Self::DuplicateProperty => None,
            Self::MissingSemicolon => {
                Some("add a semicolon `;` to the end of the property".to_string())
            }
            Self::UnexpectedArray => None,
            Self::ExpectedArray => None,
        }
    }
}

impl Display for Codes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.ident())
    }
}

impl TryFrom<&str> for Codes {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "CE1" => Ok(Self::InvalidValue),
            "CE2" => Ok(Self::InvalidValueMacro),
            "CE3" => Ok(Self::DuplicateProperty),
            "CE4" => Ok(Self::MissingSemicolon),
            "CE5" => Ok(Self::UnexpectedArray),
            "CE6" => Ok(Self::ExpectedArray),
            _ => Err(()),
        }
    }
}
