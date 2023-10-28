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
    suppress: HashMap<String, Scope>,
    flags: HashMap<String, Scope>,
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

    pub fn is_suppressed(&self, code: &str) -> bool {
        self.suppress.contains_key(code)
    }

    pub fn suppress(&mut self, token: &Rc<Token>, scope: Scope) -> Result<(), Error> {
        let code = token.symbol().to_string();
        if !["pw3_padded_arg"].contains(&code.as_str()) {
            return Err(Error::Code(Box::new(PragmaInvalidSuppress {
                token: Box::new((**token).clone()),
            })));
        }
        if let Some(existing) = self.suppress.get(&code) {
            if *existing as u8 > scope as u8 {
                return Ok(());
            }
        }
        self.suppress.insert(code, scope);
        Ok(())
    }

    pub fn is_flagged(&self, code: &str) -> bool {
        self.flags.contains_key(code)
    }

    pub fn flag(&mut self, token: &Rc<Token>, scope: Scope) -> Result<(), Error> {
        let code = token.symbol().to_string();
        if !["pw3_ignore_arr"].contains(&code.as_str()) {
            return Err(Error::Code(Box::new(PragmaInvalidFlag {
                token: Box::new((**token).clone()),
            })));
        }
        if let Some(existing) = self.flags.get(&code) {
            if *existing as u8 > scope as u8 {
                return Ok(());
            }
        }
        self.flags.insert(code, scope);
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
