use std::{collections::HashMap, rc::Rc};

use hemtt_common::reporting::Token;

use crate::{codes::pe21_pragma_invalid_suppress::PragmaInvalidSuppress, Error};

#[derive(Debug, Clone)]
pub struct Pragma {
    pub(crate) root: bool,
    supress: HashMap<String, Scope>,
}

impl Pragma {
    pub fn root() -> Self {
        Self {
            root: true,
            supress: HashMap::new(),
        }
    }

    pub fn child(&self) -> Self {
        Self {
            root: false,
            supress: {
                let mut map = HashMap::new();
                for (k, v) in &self.supress {
                    if *v as u8 == Scope::Config as u8 {
                        map.insert(k.clone(), *v);
                    }
                }
                map
            },
        }
    }

    pub fn clear_line(&mut self) {
        self.supress.retain(|_, v| *v as u8 > Scope::Line as u8);
    }

    pub fn is_suppressed(&self, code: &str) -> bool {
        self.supress.contains_key(code)
    }

    pub fn suppress(&mut self, token: &Rc<Token>, scope: Scope) -> Result<(), Error> {
        let code = token.symbol().to_string();
        if !["pw3_padded_arg"].contains(&code.as_str()) {
            return Err(Error::Code(Box::new(PragmaInvalidSuppress {
                token: Box::new((**token).clone()),
            })));
        }
        if let Some(existing) = self.supress.get(&code) {
            if *existing as u8 > scope as u8 {
                return Ok(());
            }
        }
        self.supress.insert(code, scope);
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
