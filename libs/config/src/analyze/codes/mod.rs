use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codes {
    InvalidValue,
    InvalidValueMacro,
    DuplicateProperty,
    MissingSemicolon,
}

impl Codes {
    pub const fn ident(self) -> &'static str {
        match self {
            Self::InvalidValue => "C1",
            Self::InvalidValueMacro => "C2",
            Self::DuplicateProperty => "C3",
            Self::MissingSemicolon => "C4",
        }
    }

    pub fn message(self) -> String {
        match self {
            Self::InvalidValue => "property's value could not be parsed.".to_string(),
            Self::InvalidValueMacro => "macro's result could not be parsed".to_string(),
            Self::DuplicateProperty => "property was defined more than once".to_string(),
            Self::MissingSemicolon => "property is missing a semicolon".to_string(),
        }
    }

    pub fn label_message(self) -> String {
        match self {
            Self::InvalidValue => "invalid value".to_string(),
            Self::InvalidValueMacro => "invalid macro result".to_string(),
            Self::DuplicateProperty => "duplicate property".to_string(),
            Self::MissingSemicolon => "missing semicolon".to_string(),
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
            "C1" => Ok(Self::InvalidValue),
            "C2" => Ok(Self::InvalidValueMacro),
            "C3" => Ok(Self::DuplicateProperty),
            "C4" => Ok(Self::MissingSemicolon),
            _ => Err(()),
        }
    }
}
