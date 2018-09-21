extern crate reqwest;

extern crate serde_json;
extern crate serde;

extern crate zip;

extern crate walkdir;

use colored::*;

use project;

use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Result;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Clone)]
pub struct Asset {
  pub browser_download_url: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Release {
  pub tag_name: String,
  pub assets: Vec<Asset>
}

pub fn get_releases() -> Result<Vec<Release>> {
  let body = reqwest::get("https://api.github.com/repos/KoffeinFlummi/armake/releases").unwrap().text().unwrap();
  let r: Vec<Release> = serde_json::from_str(&body).unwrap_or_else(|e| {
    panic!("Error: {}", e);
  });
  Ok(r)
}

pub fn get_latest(releases: Vec<Release>) -> Release {
  releases[0].clone()
}

pub fn download(release: &Release) -> Result<()> {
  println!("Downloading armake {}", release.tag_name);
  let mut buf: Vec<u8> = Vec::new();
  let mut req = reqwest::get(&release.assets[0].browser_download_url).unwrap();
  req.read_to_end(&mut buf);
  let mut out = File::create("armake.zip")?;
  for c in &buf {
    out.write_all(&[*c])?;
  }
  println!("Extracting");
  let file = File::open("armake.zip")?;
  let mut archive = zip::ZipArchive::new(file)?;
  for i in 0..archive.len() {
    let mut file = archive.by_index(i)?;
    let outpath = file.sanitized_name();
    extract(outpath, file)?;
  }
  fs::remove_file("armake.zip");
  let mut p = project::get_project();
  p.armake = release.tag_name.clone();
  p.save();
  Ok(())
}

#[cfg(unix)]
pub fn extract(name: PathBuf, mut source: zip::read::ZipFile) -> Result<()> {
  if !Path::new("tools").exists() {
    fs::create_dir("tools")?;
  }
  if name.ends_with("armake") {
    let mut outfile = File::create("tools/armake")?;
    io::copy(&mut source, &mut outfile)?;
    use std::os::unix::fs::PermissionsExt;
    if let Some(mode) = source.unix_mode() {
      fs::set_permissions("tools/armake", fs::Permissions::from_mode(mode)).unwrap();
    }
  }
  Ok(())
}

#[cfg(windows)]
pub fn extract(name: PathBuf, mut source: zip::read::ZipFile) -> Result<()> {
  if !Path::new("tools").exists() {
    fs::create_dir("tools")?;
  }
  if name.ends_with("armake_w64.exe") {
    let mut outfile = File::create("tools/armake.exe")?;
    io::copy(&mut source, &mut outfile)?;
  }
  Ok(())
}

pub fn modtime(addon: String) -> SystemTime {
  let mut recent: SystemTime = SystemTime::now() - Duration::new(60 * 60 * 24 * 365 * 10, 0);
  for entry in walkdir::WalkDir::new(format!("addons/{}", addon)) {
    let metadata = fs::metadata(entry.unwrap().path()).unwrap();
    if let Ok(time) = metadata.modified() {
      if time > recent {
        recent = time;
      }
    }
  }
  recent
}

pub fn build(p: &project::Project) -> Result<()> {
  for entry in fs::read_dir("addons").unwrap() {
    let entry = entry.unwrap();
    let path = entry.path();
    if !path.is_dir() {
      continue;
    }
    let cpath = path.clone();
    let cpath = cpath.to_str().unwrap().replace(r#"\"#,"/");
    let mut s = cpath.split("/");
    s.next();
    let name = s.next().unwrap().trim();
    let modified = modtime(name.to_owned());
    if Path::new(&format!("addons/{}_{}.pbo", p.prefix, name)).exists() {
      let metadata = fs::metadata(format!("addons/{}_{}.pbo", p.prefix, name)).unwrap();
      if let Ok(time) = metadata.modified() {
        if time >= modified {
          println!(" Skipping   {}", name);
          continue;
        }
      }
    }
    println!(" Building   {}", name);
    let output = Command::new("tools/armake")
            .arg("build")
            .arg("-i")
            .arg("include")
            .arg("--force")
            .arg(path)
            .arg(format!("addons/{}_{}.pbo", p.prefix, name))
            .output()?;
    if output.status.success() {
      println!(" {}      {}", "Built".green(), name.green());
    } else {
      println!(" {}     {}", "Failed".red(), name.red());
    }
    if !Path::new("logs").exists() {
      fs::create_dir("logs")?;
    }
    let mut out = File::create(format!("logs/{}.build", name))?;
    for c in output.stderr {
      out.write_all(&[c])?;
    }
  }
  Ok(())
}

pub fn release(p: &project::Project) -> Result<()> {
  let version = project::get_version();
  println!("Version: {}", version);
  if !Path::new("releases").exists() {
    fs::create_dir("releases")?;
  }
  Ok(())
}
