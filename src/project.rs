use serde::{Serialize, Deserialize};
use serde_json;

use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::io::Write;

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub prefix: String,
    pub author: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default = "get_version_unwrap")]
    pub version: Option<String>,
    #[serde(default="Vec::new")]
    pub files: Vec<String>,
    #[serde(default = "default_include")]
    pub include: Vec<PathBuf>,
    #[serde(default = "Vec::new")]
    pub exclude: Vec<String>,
    #[serde(default = "Vec::new")]
    pub optionals: Vec<String>,
}

fn default_include() -> Vec<PathBuf> {
    let mut includes = vec![];

    if PathBuf::from("./include").exists() {
        includes.push(PathBuf::from("./include"));
    }

    includes
}

impl Project {
    pub fn save(&self) -> Result<(), std::io::Error> {
        let mut out = File::create(crate::HEMTT_FILE)?;
        out.write_fmt(format_args!("{}", serde_json::to_string_pretty(&self)?))?;
        Ok(())
    }
}

pub fn init(name: String, prefix: String, author: String) -> Result<Project, std::io::Error> {
    let p = Project {
        name: name,
        prefix: prefix,
        author: author,
        version: None,
        files: vec!["mod.cpp".to_owned()],
        include: vec![],
        exclude: vec![],
        optionals: vec![],
    };
    p.save()?;
    Ok(p)
}

pub fn get_project() -> Result<Project, std::io::Error> {
    let mut f = File::open(crate::HEMTT_FILE)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    let p: Project = serde_json::from_str(contents.as_str())?;
    Ok(p)
}

pub fn get_version() -> Result<String, std::io::Error> {
    let mut version = String::from("0.0.0.0");
    if Path::new("addons/main/script_version.hpp").exists() {
        let f = BufReader::new(File::open("addons/main/script_version.hpp")?);
        let mut major = String::new();
        let mut minor = String::new();
        let mut patch = String::new();
        let mut build = String::new();
        for line in f.lines() {
            let line = line?;
            let mut split = line.split(" ");
            let define = split.next().unwrap();
            if define != "#define" { continue; }
            let key = split.next().unwrap();
            let value = split.next().unwrap().clone();
            match key {
                "MAJOR" => {
                    major = String::from(value);
                },
                "MINOR" => {
                    minor = String::from(value);
                },
                "PATCHLVL" | "PATCH" => {
                    patch = String::from(value);
                },
                "BUILD" => {
                    build = String::from(value);
                },
                _ => {}
            }
        }
        if build == "" {
            version = format!("{}.{}.{}", major, minor, patch);
        } else {
            version = format!("{}.{}.{}.{}", major, minor, patch, build);
        }
    }
    Ok(version)
}
fn get_version_unwrap() -> Option<String> {
    Some(get_version().unwrap())
}
