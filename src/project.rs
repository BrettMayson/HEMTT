extern crate serde;
extern crate serde_json;

use armake;

use std::fs::File;
use std::io::BufReader;
use std::io::Result;
use std::io::prelude::*;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct Project {
  pub name: String,
  pub prefix: String,
  pub author: String,
  pub files: Vec<String>
}

impl Project {
  pub fn save(&self) -> Result<()> {
    let mut out = File::create(::HEMTT_FILE).expect("Unable to create output file");
    out.write_fmt(format_args!("{}", serde_json::to_string_pretty(&self)?));
    Ok(())
  }
}

pub fn get_project() -> Project {
  let mut f = File::open(::HEMTT_FILE).expect("file not found");
  let mut contents = String::new();
  f.read_to_string(&mut contents).expect("something went wrong reading the file");
  let p: Project = serde_json::from_str(contents.as_str()).expect("Error parsing data");
  p
}

pub fn create(name: String, prefix: String, author: String) -> Project {
  let releases = armake::get_releases().unwrap();
  let latest = armake::get_latest(releases);
  let p = Project {
    name: name,
    prefix: prefix,
    author: author,
    files: vec!["mod.cpp".to_owned()]
  };
  p.save();
  p
}


pub fn get_version() -> String {
  let mut version = String::from("0.0.0.0");
  if Path::new("addons/main/script_version.hpp").exists() {
    let f = BufReader::new(File::open("addons/main/script_version.hpp").expect("open failed"));
    let mut major = String::new();
    let mut minor = String::new();
    let mut patch = String::new();
    let mut build = String::new();
    for line in f.lines() {
      let line = line.expect("lines failed");
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
        "PATCHLVL" => {
          patch = String::from(value);
        },
        "BUILD" => {
          build = String::from(value);
        },
        _ => {}
      }
    }
    version = format!("{}.{}.{}.{}", major, minor, patch, build);
  }
  version
}
