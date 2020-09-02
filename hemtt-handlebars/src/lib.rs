use std::collections::BTreeMap;

use handlebars::*;
use serde_json::value::Value as Json;

mod helpers;

pub struct Variables(BTreeMap<String, Json>);
impl Variables {
    pub fn inner(&self) -> &BTreeMap<String, Json> {
        &self.0
    }
    pub fn append(&mut self, mut other: BTreeMap<String, Json>) {
        self.0.append(&mut other);
    }
    pub fn insert<S: Into<String>>(&mut self, key: S, value: Json) {
        self.0.insert(key.into(), value);
    }
}

pub fn render(source: &str, data: &Variables) -> Result<String, TemplateRenderError> {
    let mut handlebars = Handlebars::new();
    handlebars.register_helper("date", Box::new(helpers::date));
    handlebars.register_helper("git", Box::new(helpers::git));
    handlebars.set_strict_mode(true);
    handlebars.render_template(source, data.inner())
}

impl From<semver::Version> for Variables {
    fn from(version: semver::Version) -> Self {
        Self({
            let mut map = BTreeMap::new();
            map.insert(
                String::from("semver"),
                serde_json::Value::Object({
                    let mut map = serde_json::Map::new();
                    map.insert(
                        String::from("major"),
                        serde_json::Value::Number(serde_json::value::Number::from(version.major)),
                    );
                    map.insert(
                        String::from("minor"),
                        serde_json::Value::Number(serde_json::value::Number::from(version.minor)),
                    );
                    map.insert(
                        String::from("patch"),
                        serde_json::Value::Number(serde_json::value::Number::from(version.patch)),
                    );
                    map
                }),
            );
            map
        })
    }
}
