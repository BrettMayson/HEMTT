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

pub fn get_installed() -> String {
  if Path::new("tools/armake.version").exists() {
    let mut f = File::open("tools/armake.version").expect("version file not found");
    let mut contents = String::new();
    f.read_to_string(&mut contents).expect("something went wrong reading the version file");
    contents.trim().to_owned()
  } else {
    "".to_owned()
  }
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
  let mut out = File::create("tools/armake.version").expect("Unable to create version file");
  out.write_fmt(format_args!("{}", release.tag_name));
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
  build(&p);
  let version = project::get_version();
  println!("Version: {}", version);
  if !Path::new("releases").exists() {
    fs::create_dir("releases")?;
  }
  if !Path::new(&format!("releases/{}", version)).exists() {
    fs::create_dir(format!("releases/{}", version))?;
  }
  if !Path::new(&format!("releases/{}/@{}", version, p.prefix)).exists() {
    fs::create_dir(format!("releases/{}/@{}", version, p.prefix))?;
  }
  if !Path::new(&format!("releases/{}/@{}/addons", version, p.prefix)).exists() {
    fs::create_dir(format!("releases/{}/@{}/addons", version, p.prefix))?;
  }
  if !Path::new(&format!("releases/{}/@{}/keys", version, p.prefix)).exists() {
    fs::create_dir(format!("releases/{}/@{}/keys", version, p.prefix))?;
  }
  for file in &p.files {
    fs::copy(file, format!("releases/{}/@{}/{}", version, p.prefix, file));
  }
  if !Path::new("keys").exists() {
    fs::create_dir("keys")?;
  }
  if !Path::new(&format!("keys/{}.bikey", p.prefix)).exists() {
    let output = Command::new("tools/armake")
            .arg("keygen")
            .arg(&p.prefix)
            .output()?;
    fs::rename(format!("{}.bikey", p.prefix), format!("keys/{}.bikey", p.prefix));
    fs::rename(format!("{}.biprivatekey", p.prefix), format!("keys/{}.biprivatekey", p.prefix));
  }
  fs::copy(format!("keys/{}.bikey", p.prefix), format!("releases/{0}/@{1}/keys/{1}.bikey", version, p.prefix));
  for entry in fs::read_dir("addons").unwrap() {
    let entry = entry.unwrap();
    let path = entry.path();
    let cpath = path.clone();
    let cpath = cpath.to_str().unwrap().replace(r#"\"#,"/");
    if !path.ends_with(".pbo") && !cpath.contains(p.prefix.as_str()) {
      continue;
    }
    println!("{}", cpath);
    fs::copy(&cpath, format!("releases/{}/@{}/{}", version, p.prefix, cpath));
    let output = Command::new("tools/armake")
            .arg("sign")
            .arg("-s")
            .arg(format!("releases/{}/@{}/{}", version, p.prefix, cpath))
            .arg(format!("keys/{}.biprivatekey", p.prefix))
            .arg(format!("releases/{}/@{}/{}", version, p.prefix, cpath))
            .output()?;
  }
  Ok(())
}
