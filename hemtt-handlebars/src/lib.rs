#[macro_use]
extern crate log;

use std::collections::BTreeMap;

use handlebars::*;
use serde_json::value::Value as Json;

mod helpers;

pub fn render(source: &str, data: &Variables) -> Result<String, RenderError> {
    let mut handlebars = Handlebars::new();
    handlebars.register_helper("date", Box::new(helpers::date));
    handlebars.register_helper("git", Box::new(helpers::git));
    handlebars.set_strict_mode(true);
    handlebars.render_template(source, data.inner())
}

#[derive(Default)]
pub struct Variables(BTreeMap<String, Json>);
impl Variables {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
    pub fn inner(&self) -> &BTreeMap<String, Json> {
        &self.0
    }
    pub fn append(&mut self, mut other: Variables) {
        self.0.append(&mut other.0);
    }
    pub fn insert<S: Into<String>>(&mut self, key: S, value: Json) {
        self.0.insert(key.into(), value);
    }
}

impl From<BTreeMap<String, Json>> for Variables {
    fn from(map: BTreeMap<String, Json>) -> Self {
        Self(map)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::value::Value as Json;
    use std::collections::BTreeMap;

    use crate::{render, Variables};

    #[test]
    fn variables() {
        let mut map = BTreeMap::<String, Json>::new();
        map.insert("a".to_string(), Json::String(String::from("1")));
        let mut var = Variables::from(map);
        var.insert("b", Json::String(String::from("2")));
        let mut map2 = BTreeMap::<String, Json>::new();
        map2.insert("c".to_string(), Json::String(String::from("3")));
        var.append(Variables::from(map2));
        assert_eq!(render("{{a}}{{b}}{{c}}", &var).unwrap(), "123");
    }
}
