use std::{collections::HashMap, rc::Rc};

use hemtt_common::reporting::Token;

use crate::{
    codes::{
        pe21_pragma_invalid_suppress::PragmaInvalidSuppress,
        pe22_pragma_invalid_flag::PragmaInvalidFlag,
    },
    Error,
};

#[derive(Debug, Clone)]
pub struct Pragma {
    pub(crate) root: bool,
    suppress: HashMap<Suppress, Scope>,
    flags: HashMap<Flag, Scope>,
}

impl Pragma {
    pub fn root() -> Self {
        Self {
            root: true,
            suppress: HashMap::new(),
            flags: HashMap::new(),
        }
    }

    pub fn child(&self) -> Self {
        Self {
            root: false,
            suppress: {
                let mut map = HashMap::new();
                for (k, v) in &self.suppress {
                    if *v as u8 == Scope::Config as u8 {
                        map.insert(k.clone(), *v);
                    }
                }
                map
            },
            flags: {
                let mut map = HashMap::new();
                for (k, v) in &self.flags {
                    if *v as u8 == Scope::Config as u8 {
                        map.insert(k.clone(), *v);
                    }
                }
                map
            },
        }
    }

    pub fn clear_line(&mut self) {
        self.suppress.retain(|_, v| *v as u8 > Scope::Line as u8);
    }

    pub fn is_suppressed(&self, code: &Suppress) -> bool {
        self.suppress.contains_key(code)
    }

    pub fn suppress(&mut self, token: &Rc<Token>, scope: Scope) -> Result<(), Error> {
        let code = token.symbol().to_string();
        let Ok(suppress) = Suppress::try_from(code.as_str()) else {
            return Err(PragmaInvalidSuppress::code((**token).clone()));
        };
        if let Some(existing) = self.suppress.get(&suppress) {
            if *existing as u8 > scope as u8 {
                return Ok(());
            }
        }
        self.suppress.insert(suppress, scope);
        Ok(())
    }

    pub fn is_flagged(&self, code: &Flag) -> bool {
        self.flags.contains_key(code)
    }

    pub fn flag(&mut self, token: &Rc<Token>, scope: Scope) -> Result<(), Error> {
        let code = token.symbol().to_string();
        let Ok(flag) = Flag::try_from(code.as_str()) else {
            return Err(PragmaInvalidFlag::code((**token).clone()));
        };
        if let Some(existing) = self.flags.get(&flag) {
            if *existing as u8 > scope as u8 {
                return Ok(());
            }
        }
        self.flags.insert(flag, scope);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scope {
    Line,
    File,
    Config,
}

impl TryFrom<&str> for Scope {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "line" => Ok(Self::Line),
            "file" => Ok(Self::File),
            "config" => Ok(Self::Config),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Flag {
    Pw3IgnoreFormat,
    Pe23IgnoreIfHasInclude,
}

impl Flag {
    pub const fn as_slice() -> &'static [&'static str] {
        &["pw3_ignore_format", "pe23_ignore_has_include"]
    }
}

impl TryFrom<&str> for Flag {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pw3_ignore_format" => Ok(Self::Pw3IgnoreFormat),
            "pe23_ignore_has_include" => Ok(Self::Pe23IgnoreIfHasInclude),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Suppress {
    Pw3PaddedArg,
}

impl Suppress {
    pub const fn as_slice() -> &'static [&'static str] {
        &["pw3_padded_arg"]
    }
}

impl TryFrom<&str> for Suppress {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pw3_padded_arg" => Ok(Self::Pw3PaddedArg),
            _ => Err(()),
        }
    }
}
