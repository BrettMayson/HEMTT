use std::{path::Path, sync::Arc};

use hemtt_common::{
    reporting::{simple, Code},
    similar_values,
};

use crate::Error;

pub struct ScriptNotFound {
    script: String,
    similar: Vec<String>,
}

impl Code for ScriptNotFound {
    fn ident(&self) -> &'static str {
        "BHE1"
    }

    fn message(&self) -> String {
        format!("Script not found: {}", self.script)
    }

    fn help(&self) -> Option<String> {
        if self.similar.is_empty() {
            None
        } else {
            Some(format!("did you mean `{}`?", self.similar.join("`, `")))
        }
    }

    fn report(&self) -> Option<String> {
        Some(simple(self, ariadne::ReportKind::Error, self.help()))
    }

    fn ci(&self) -> Vec<hemtt_common::reporting::Annotation> {
        Vec::new()
    }
}

impl ScriptNotFound {
    pub fn code(script: String, path: &Path) -> Result<Arc<dyn Code>, Error> {
        let scripts = path
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
                &script,
                &scripts
                    .iter()
                    .map(std::string::String::as_str)
                    .collect::<Vec<&str>>(),
            )
            .iter()
            .map(std::string::ToString::to_string)
            .collect(),
            script,
        }))
    }
}
