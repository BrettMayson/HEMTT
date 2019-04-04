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
use crate::dft_false;

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

    #[serde(default = "dft_false")]
    pub reuse_private_key: bool,

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
        let mut out = File::create(&file)?;
        match file.extension().unwrap().to_str().unwrap() {
            "toml" => out.write_fmt(format_args!("{}", toml::to_string(&self).unwrap()))?,
            "json" => out.write_fmt(format_args!("{}", serde_json::to_string_pretty(&self)?))?,
            _ => unreachable!()
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
            if self.reuse_private_key {
                self.prefix.clone()
            } else {
                format!("{}_{}", &self.prefix, &self.version.clone().unwrap())
            }
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
        reuse_private_key: false,

        template_data: BTreeMap::new(),
    };
    p.save()?;
    Ok(p)
}

pub fn exists() -> Result<PathBuf, Error> {
    let toml = toml_file();
    if toml.is_ok() {
        return toml;
    }
    json_file()
}

pub fn path(fail: bool) -> Result<PathBuf, Error> {
    let file = exists();
    if file.is_ok() {
        return Ok(file.unwrap());
    } else if !fail {
        return Ok(PathBuf::from("./hemtt.json"));
    }
    Err(error!("No HEMTT project file was found"))
}

pub fn json_file() -> Result<PathBuf, Error> {
    search_for("hemtt.json")
}

pub fn toml_file() -> Result<PathBuf, Error> {
    search_for("hemtt.toml")
}

pub fn search_for(s: &'static str) -> Result<PathBuf, Error> {
    let mut dir = std::env::current_dir().unwrap_or_print();
    loop {
        let mut search = dir.clone();
        search.push(s.clone());
        if search.exists() {
            return Ok(search);
        }
        dir.pop();
        search.pop();
        if dir == search {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No HEMTT Project File was found"));
        }
    }
}

pub fn get_project() -> Result<Project, Error> {
    let file = path(true)?;
    std::env::set_current_dir(file.parent().unwrap()).unwrap_or_print();
    let mut f = File::open(&file)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    let mut p: Project = match file.extension().unwrap().to_str().unwrap() {
        "toml" => toml::from_str(contents.as_str()).unwrap_or_print(),
        "json" => serde_json::from_str(contents.as_str())?,
        _ => unreachable!()
    };
    p.template_data = BTreeMap::new();
    p.template_data.insert("name", p.name.clone());
    p.template_data.insert("prefix", p.prefix.clone());
    p.template_data.insert("author", p.author.clone());
    p.template_data.insert("version", p.version.clone().unwrap());
    Ok(p)
}

pub fn use_project_dir() {
    let file = path(true).unwrap_or_print();
    std::env::set_current_dir(file.parent().unwrap()).unwrap_or_print();
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
