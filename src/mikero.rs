extern crate winreg;
use winreg::enums::*;
use winreg::RegKey;

use colored::*;

use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use files;
use project;

pub struct toolchain {
  pub pboProject: String,
  pub rapify: String,
  pub makePbo: String
}

pub fn toolchain() -> Result<toolchain, io::Error> {
  let hkcu = RegKey::predef(HKEY_CURRENT_USER);
  let software = hkcu.open_subkey("Software\\Mikero")?;
  //pboProject
  Ok(toolchain {
    pboProject: software.open_subkey("pboProject")?.get_value("exe")?,
    rapify: software.open_subkey("rapify")?.get_value("exe")?,
    makePbo: software.open_subkey("makePbo")?.get_value("exe")?
  })
}

pub fn create_pdrive(p: &project::Project) -> Result<(), io::Error> {
  Command::new("subst")
          .arg("p:")
          .arg(&p.pdrive)
          .output()?;
  Ok(())
}

pub fn remove_pdrive() -> Result<(), io::Error> {
  Command::new("subst")
          .arg("p:")
          .arg("/d")
          .output()?;
  Ok(())
}

impl toolchain {
  pub fn build(&self, p: &project::Project) -> Result<(), io::Error> {
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
      let modified = files::modtime(name.to_owned());
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
      let output = Command::new(&self.makePbo)
              .arg("-NUP")
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

  pub fn release(&self, p: &project::Project) -> Result<(), io::Error> {
    let version = project::get_version();
    println!("Building Release Version: {}", version);
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
    /*if !Path::new(&format!("keys/{}.bikey", p.prefix)).exists() {
      let output = Command::new("tools/armake")
              .arg("keygen")
              .arg(&p.prefix)
              .output()?;
      fs::rename(format!("{}.bikey", p.prefix), format!("keys/{}.bikey", p.prefix));
      fs::rename(format!("{}.biprivatekey", p.prefix), format!("keys/{}.biprivatekey", p.prefix));
    }*/
    fs::copy(format!("keys/{}.bikey", p.prefix), format!("releases/{0}/@{1}/keys/{1}.bikey", version, p.prefix));
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
      fs::copy(&cpath, format!("releases/{}/@{}/{}", version, p.prefix, cpath));
      let output = Command::new(&self.makePbo)
              .arg("-P")
              .arg("-A")
              .arg("-G")
              .arg(path)
              .arg(format!("releases/{}/@{}/{}", version, p.prefix, cpath))
              .output()?;
      if output.status.success() {
        println!(" {}     {}", "Signed".green(), cpath.green());
      } else {
        println!(" {} {}", "Not Signed".red(), cpath.red());
      }
      if !Path::new("logs").exists() {
        fs::create_dir("logs")?;
      }
      let mut out = File::create(format!("logs/{}.build.release", name))?;
      for c in output.stderr {
        out.write_all(&[c])?;
      }
    }
    Ok(())
  }
}
