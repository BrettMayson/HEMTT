use std::collections::BTreeMap;

use handlebars::*;
use serde_json::value::Value as Json;

pub mod helpers;

use crate::error::*;

pub fn run(source: &str, filename: Option<&str>, data: &BTreeMap<&'static str, Json>) -> Result<String, HEMTTError> {
    let mut handlebars = Handlebars::new();
    handlebars.register_helper("date", Box::new(helpers::date));
    handlebars.register_helper("git", Box::new(helpers::git));
    handlebars.set_strict_mode(true);
    handlebars.render_template(&source, data).map_err(|err| match err {
        handlebars::TemplateRenderError::RenderError(e) => {
            if e.line_no.is_some() {
                HEMTTError::LINENO(FileErrorLineNumber {
                    error: e.desc,
                    line: e.line_no,
                    col: e.column_no,
                    note: None,
                    file: filename.unwrap_or("").to_string(),
                    content: source.to_owned(),
                })
            } else {
                HEMTTError::GENERIC("Render error".to_string(), e.desc)
            }
        }
        handlebars::TemplateRenderError::TemplateError(e) => {
            if e.line_no.is_some() {
                HEMTTError::LINENO(FileErrorLineNumber {
                    error: e.reason.to_string(),
                    line: e.line_no,
                    col: e.column_no,
                    note: None,
                    file: filename.unwrap_or("").to_string(),
                    content: source.to_owned(),
                })
            } else {
                HEMTTError::GENERIC("Render error".to_string(), e.reason.to_string())
            }
        }
        _ => unimplemented!(),
    })
}
