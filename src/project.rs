use serde::{Serialize, Deserialize};
use serde_json;
use toml;

use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{Write, Error};
use std::io::BufReader;
use std::io::prelude::*;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::*;
use crate::template::render;
use crate::state::State;

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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "default_include")]
    pub include: Vec<PathBuf>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub exclude: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub optionals: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub skip: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub headerexts: Vec<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default = "String::new")]
    pub modname: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default = "String::new")]
    pub keyname: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default = "String::new")]
    pub signame: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub prebuild: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub postbuild: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub releasebuild: Vec<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default = "HashMap::new")]
    pub scripts: HashMap<String, crate::build::script::BuildScript>,

    #[serde(skip_deserializing,skip_serializing)]
    pub template_data: BTreeMap<&'static str, String>,
}

fn default_include() -> Vec<PathBuf> {
    let mut includes = vec![];

    if PathBuf::from("./include").exists() {
        includes.push(PathBuf::from("./include"));
    }

    includes
}

impl Project {
    pub fn save(&self) -> Result<(), Error> {
        let file = path(false).unwrap_or_print();
        let mut out = File::create(file)?;
        if toml_exists() {
            out.write_fmt(format_args!("{}", toml::to_string(&self).unwrap()))?;
        } else {
            out.write_fmt(format_args!("{}", serde_json::to_string_pretty(&self)?))?;
        }
        Ok(())
    }

    pub fn get_modname(&self) -> String {
        if self.modname.is_empty() {
            self.prefix.clone()
        } else {
            render(&self.modname, &self.template_data)
        }
    }

    pub fn get_keyname(&self) -> String {
        if self.keyname.is_empty() {
            self.prefix.clone()
        } else {
            render(&self.keyname, &self.template_data)
        }
    }

    pub fn get_signame(&self, pbo: &String) -> String {
        if self.signame.is_empty() {
            format!("{}.{}.bisign", pbo, &self.version.clone().unwrap())
        } else {
            format!("{}.{}.bisign", pbo, render(&self.signame, &self.template_data))
        }
    }

    pub fn get_headerexts(&self) -> Vec<String> {
        let mut headerexts = self.headerexts.clone();
        for headerext in headerexts.iter_mut() {
            *headerext = render(&headerext, &self.template_data);
        }
        headerexts
    }

    pub fn run(&self, state: &State) -> Result<(), Error> {
        crate::build::script::run(&self, &state)
    }

    pub fn script(&self, name: &String, state: &State) -> Result<(), Error> {
        if self.scripts.contains_key(name) {
            let script = self.scripts.get(name).unwrap();
            if script.foreach && state.stage == crate::state::Stage::Script {
                println!("Unable to run scripts with 'foreach' outside of build steps");
                std::process::exit(1);
            } else {
                script.run(&self, &state).unwrap_or_print();
            }
        } else {
            return Err(error!("Undefined script: {}", &name));
        }
        Ok(())
    }

    pub fn render(&self, text: &String) -> String {
        crate::template::render(text, &self.template_data)
    }
}

pub fn init(name: String, prefix: String, author: String) -> Result<Project, Error> {
    let p = Project {
        name: name,
        prefix: prefix,
        author: author,
        version: None,
        files: vec!["mod.cpp".to_owned()],
        include: Vec::new(),
        exclude: Vec::new(),
        optionals: Vec::new(),
        skip: Vec::new(),
        headerexts: Vec::new(),
        modname: String::new(),
        keyname: String::new(),
        signame: String::new(),
        prebuild: Vec::new(),
        postbuild: Vec::new(),
        releasebuild: Vec::new(),
        scripts: HashMap::new(),

        template_data: BTreeMap::new(),
    };
    p.save()?;
    Ok(p)
}

pub fn exists() -> bool {
    toml_exists() || json_exists()
}

pub fn path(fail: bool) -> Result<&'static Path, Error> {
    if exists() {
        return Ok(Path::new(
            if toml_exists() {"hemtt.toml"} else {"hemtt.json"}
        ));
    } else if !fail {
        return Ok(Path::new("hemtt.json"));
    }
    Err(error!("No HEMTT project file was found"))
}

pub fn json_exists() -> bool {
    Path::new("hemtt.json").exists()
}

pub fn toml_exists() -> bool {
    Path::new("hemtt.toml").exists()
}

pub fn get_project() -> Result<Project, Error> {
    let file = path(true)?;
    let mut f = File::open(file)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    let mut p: Project = match toml_exists() {
        true => toml::from_str(contents.as_str()).unwrap_or_print(),
        false => serde_json::from_str(contents.as_str())?
    };
    p.template_data = BTreeMap::new();
    p.template_data.insert("name", p.name.clone());
    p.template_data.insert("prefix", p.prefix.clone());
    p.template_data.insert("author", p.author.clone());
    p.template_data.insert("version", p.version.clone().unwrap());
    Ok(p)
}

pub fn get_version() -> Result<String, Error> {
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
