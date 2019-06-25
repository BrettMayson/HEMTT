use std::env;
use std::path::PathBuf;
use std::collections::BTreeMap;

use config::{ConfigError, Config, File, Environment};
use handlebars::to_json;
use serde_json::value::{Value as Json};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub prefix: String,
    pub author: String,
    pub template: String,

    #[serde(default = "default_mainprefix")]
    pub mainprefix: String,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "default_include")]
    pub include: Vec<PathBuf>,

    #[serde(skip_deserializing, skip_serializing)]
    #[serde(default = "default_rendered_files")]
    pub rendered_files: crate::build::prebuild::render::RenderedFiles,
}
impl Project {
    pub fn new(name: String, prefix: String, author: String, template: String) -> Self {
        Self {
            name, prefix, author, template,

            mainprefix: default_mainprefix(),
            include: default_include(),

            rendered_files: default_rendered_files(),
        }
    }

    pub fn read() -> Result<Self, ConfigError> {
        let mut p = Config::new();
        let env = environment();
        p.merge(File::with_name(&format!("hemtt/{}", env)).required(false));
        p.merge(File::with_name("hemtt/local").required(false));
        p.merge(Environment::with_prefix("app"))?;
        p.try_into()
    }

    pub fn get_variables(&self) -> BTreeMap<&'static str, Json> {
        let mut vars = BTreeMap::new();
        vars.insert("name", to_json(self.name.clone()));
        vars.insert("prefix", to_json(self.prefix.clone()));
        vars.insert("mainprefix", to_json(self.mainprefix.clone()));
        vars.insert("author", to_json(self.author.clone()));
        vars.insert("env", to_json(environment()));
        vars
    }
}

pub fn environment() -> String {
    env::var("ENV").unwrap_or_else(|_| "dev".into())
}

#[derive(Serialize, Deserialize)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub build: String,
}
impl SemVer {
    pub fn new(major: u32, minor: u32, patch: u32, build: String) -> Self {
        SemVer { major, minor, patch, build }
    }
    pub fn to_string(&self) -> String {
        if self.build.is_empty() {
            format!("{}.{}.{}", self.major, self.minor, self.patch)
        } else {
            format!("{}.{}.{}.{}", self.major, self.minor, self.patch, self.build)
        }
    }
}

fn default_include() -> Vec<PathBuf> {
    let mut includes = vec![];

    if PathBuf::from("./include").exists() {
        includes.push(PathBuf::from("./include"));
    }

    includes
}

fn default_mainprefix() -> String {
    String::from("z")
}

fn default_rendered_files() -> crate::RenderedFiles {
    crate::RenderedFiles::new()
}
