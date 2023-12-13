use std::{path::Path, sync::Arc};

use hemtt_common::{
    reporting::{simple, Code},
    similar_values,
};

use crate::Error;

pub struct PresetNotFound {
    name: String,
    similar: Vec<String>,
}
impl Code for PresetNotFound {
    fn ident(&self) -> &'static str {
        "BCLE1"
    }

    fn message(&self) -> String {
        format!("Preset `{}` not found.", self.name)
    }

    fn help(&self) -> Option<String> {
        if self.similar.is_empty() {
            None
        } else {
            Some(format!("Did you mean `{}`?", self.similar.join("`, `")))
        }
    }

    fn report(&self) -> Option<String> {
        Some(simple(self, ariadne::ReportKind::Error, self.help()))
    }

    fn ci(&self) -> Vec<hemtt_common::reporting::Annotation> {
        vec![]
    }
}

impl PresetNotFound {
    pub fn code(name: String, path: &Path) -> Result<Arc<dyn Code>, Error> {
        let presets = path
            .read_dir()?
            .filter_map(|x| {
                x.ok().and_then(|x| {
                    if x.file_type().ok()?.is_file() {
                        x.file_name().to_str().map(std::borrow::ToOwned::to_owned)
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<String>>();
        Ok(Arc::new(Self {
            similar: similar_values(
                &name,
                &presets
                    .iter()
                    .map(std::string::String::as_str)
                    .collect::<Vec<&str>>(),
            )
            .iter()
            .map(std::string::ToString::to_string)
            .collect(),
            name,
        }))
    }
}
