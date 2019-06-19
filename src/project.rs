use std::env;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use config::{ConfigError, Config, File, Environment};

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub prefix: String,
    pub author: String,
    pub template: String,
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

impl Project {
    pub fn read() -> Result<Self, ConfigError> {
        let mut p = Config::new();
        let env = env::var("MODE").unwrap_or_else(|_| "dev".into());
        p.merge(File::with_name(&format!("hemtt/{}", env)).required(false));
        p.merge(File::with_name("hemtt/local").required(false));
        p.merge(Environment::with_prefix("app"))?;
        p.try_into()
    }
}
