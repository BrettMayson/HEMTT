use walkdir;
use armake2;

use std::fs;
use std::fs::File;
use std::time::{Duration, SystemTime};
use std::path::{Path, PathBuf};

pub fn modtime(addon: String) -> Result<SystemTime, std::io::Error> {
  let mut recent: SystemTime = SystemTime::now() - Duration::new(60 * 60 * 24 * 365 * 10, 0);
  for entry in walkdir::WalkDir::new(format!("addons/{}", addon)) {
    let metadata = fs::metadata(entry.unwrap().path())?;
    if let Ok(time) = metadata.modified() {
      if time > recent {
        recent = time;
      }
    }
  }
  Ok(recent)
}

pub fn build(p: &crate::project::Project) -> Result<(), std::io::Error> {
  for entry in fs::read_dir("addons")? {
    let entry = entry?;
    let path = entry.path();
    if !path.is_dir() { continue };
    let cpath = path.clone().to_str().unwrap().replace(r#"\"#,"/");
    let mut s = cpath.split("/");
    s.next();
    let name = s.next().unwrap().trim();
    let modified = modtime(name.to_owned())?;
    if Path::new(&format!("addons/{}_{}.pbo", p.prefix, name)).exists() {
      let metadata = fs::metadata(format!("addons/{}_{}.pbo", p.prefix, name)).unwrap();
      if let Ok(time) = metadata.modified() {
        if time >= modified {
          println!(" Skipping   {}", name);
          continue;
        }
      }
    }
    println!("Building {}", name);
    let mut outf = File::create(&format!("addons/{}_{}.pbo", p.prefix, name))?;
    armake2::pbo::cmd_build(
      path,
      &mut outf,
      &vec![],
      &vec![],
      &vec![PathBuf::from("./include"), PathBuf::from(".")],
    )?;
  }
  Ok(())
}