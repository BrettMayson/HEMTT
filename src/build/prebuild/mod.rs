use std::collections::{HashMap};
use std::path::Path;

pub mod preprocess;
pub mod render;

use crate::HEMTTError;

#[derive(Default, Clone)]
pub struct RenderedFiles {
    redirects: HashMap<String, String>,
    pub no_drop: bool,
}

impl RenderedFiles {
    pub fn new() -> Self {
        Self {
            redirects: HashMap::new(),
            no_drop: false,
        }
    }

    pub fn add(&mut self, original: String, tmp: String) -> Result<(), HEMTTError> {
        self.redirects.insert(original.clone(), tmp.clone());
        Ok(())
    }

    pub fn get_path(&self, original: String) -> Option<&String> {
        self.redirects.get(&original)
    }

    pub fn get_paths(&self, original: String) -> (String, String) {
        let rendered = crate::build::prebuild::render::can_render(&Path::new(&original));
        if rendered {
            (original.replace(".ht.", ".").trim_end_matches(".ht").to_string(), self.redirects.get(&original).unwrap().to_string())
        } else {
            (original.clone(), original)
        }
    }
}

impl RenderedFiles {
    pub fn clean(&mut self) {
        for (_, tmp) in self.redirects.iter() {
            if let Err(e) = std::fs::remove_file(tmp) {
                error!(e.to_string());
            }
        }
    }
}
