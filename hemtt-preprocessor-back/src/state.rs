use std::collections::HashMap;
use std::convert::TryFrom;

use super::Define;

#[derive(Default, Clone)]
pub struct State(HashMap<String, Define>);

impl State {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, ident: String, define: Define) -> Result<(), ()> {
        self.0.insert(ident, define);
        Ok(())
    }

    pub fn defines(&mut self) -> &mut HashMap<String, Define> {
        &mut self.0
    }

    pub fn get(&mut self, key: &str) -> Option<Define> {
        if let Some(d) = self.0.get(key) {
            Some(d.to_owned())
        } else {
            None
        }
    }

    pub fn include(&mut self, sections: Vec<&str>) -> Result<(), ()> {
        unimplemented!()
    }

    pub fn define(&mut self, line: String) -> Result<(), ()> {
        let d = Define::try_from(line)?;
        if self.0.remove(&d.ident).is_some() {
            debug!("redefine: {}", d.ident);
        } else {
            debug!("define: {}", d.ident);
        };
        println!("Setting define: `{}`", d.ident);
        self.0.insert(d.ident.clone(), d);
        Ok(())
    }

    pub fn undefine(&mut self, sections: Vec<&str>) -> Result<(), ()> {
        if let Some(ident) = sections.get(1) {
            debug!("undef {}", ident);
            if if let Some(index) = ident.find('(') {
                self.0.remove(&ident[..index])
            } else {
                self.0.remove(*ident)
            }
            .is_none()
            {
                warn!("undef {} not exist", ident);
            }
        } else {
            warn!("undef empty");
        }
        Ok(())
    }

    pub fn _if(&self, sections: Vec<&str>) -> Result<(), ()> {
        unimplemented!()
    }
    pub fn _else(&self, sections: Vec<&str>) -> Result<(), ()> {
        unimplemented!()
    }
    pub fn _end(&self, sections: Vec<&str>) -> Result<(), ()> {
        unimplemented!()
    }
}
