extern crate serde;
extern crate serde_json;

use armake;

use std::fs::File;
use std::io::prelude::*;
use std::io::Result;

#[derive(Serialize, Deserialize)]
pub struct Project {
  pub name: String,
  pub prefix: String,
  pub author: String,
  pub armake: String
}

impl Project {
  pub fn save(&self) -> Result<()> {
    save_project(&self.name, &self.prefix, &self.author, &self.armake)?;
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

pub fn save_project(name: &String, prefix: &String, author: &String, armake: &String) -> Result<()> {
  let mut out = File::create(::HEMTT_FILE).expect("Unable to create output file");
  out.write_fmt(
    format_args!("{{\n  \"name\": \"{}\",\n  \"prefix\": \"{}\",\n  \"author\": \"{}\",\n  \"armake\": \"{}\"\n}}",
      name,
      prefix,
      author,
      armake
    )
  ).expect("Error writing to file");
  Ok(())
}

pub fn create(name: String, prefix: String, author: String) -> Project {
  let releases = armake::get_releases().unwrap();
  let latest = armake::get_latest(releases);
  save_project(&name, &prefix, &author, &latest.tag_name);
  Project {
    name: name,
    prefix: prefix,
    author: author,
    armake: latest.tag_name
  }
}


pub fn get_version() -> Result<String> {
  if Path::new("addons/main/script_version.hpp").exists() {
    let f = BufReader::new(File::open("addons/main/script_version.hpp").expect("open failed"));
    for line in f.lines() {
      let line = line.expect("lines failed")
    }
  } else {
    Err("script_version.hpp does not exist");
  }
}
