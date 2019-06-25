use std::collections::BTreeMap;

use handlebars::*;
use serde_json::value::{Value as Json};

pub mod helpers;

use crate::error::*;

pub fn run(source: &str, data: &BTreeMap<&'static str, Json>) -> Result<String, HEMTTError> {
    let mut handlebars = Handlebars::new();
    handlebars.register_helper("date", Box::new(helpers::date));
    handlebars.register_helper("git", Box::new(helpers::git));
    handlebars.set_strict_mode(true);
    handlebars.render_template(&source, data).map_err(From::from)
}
