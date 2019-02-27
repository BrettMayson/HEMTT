use handlebars::Handlebars;

use std::collections::BTreeMap;

use crate::error::*;

pub fn render(source: &String, data: &BTreeMap<&'static str, String>) -> String {
    let handlebars = Handlebars::new();
    handlebars.render_template(&source, &data).unwrap_or_print()
}
