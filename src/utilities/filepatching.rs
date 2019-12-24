use std::io::Read;
use std::path::PathBuf;

use regex::Regex;
use winreg::enums::*;
use winreg::RegKey;

use crate::{Command, HEMTTError, Project};

pub struct FilePatching {}
impl FilePatching {
    pub fn create_link(dir: PathBuf, p: &Project) -> Result<(), HEMTTError> {
        let mut dir = dir;
        dir.push(&p.mainprefix);
        let mut target = dir.clone();
        target.push(&p.modname()?);
        if target.exists() {
            return Err(HEMTTError::simple(format!("The link already exists at {:?}", dir)));
        } else {
            create_dir!(dir)?;
            let project_root = crate::project::find_root()?.to_str().unwrap().to_string();
            std::env::set_current_dir(dir)?;
            std::process::Command::new("cmd")
                .args(&["/C", "mklink", "/J", &p.modname()?, &project_root])
                .output()?;
            println!("Linked at {:?}", target);
            println!(
                "You can now use `-mod=\"\\{}\\{}\" -filePatching`",
                p.mainprefix,
                p.modname()?
            );
        }
        Ok(())
    }
}
impl Command for FilePatching {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("file-patching")
            .about("Setup file patching links using `mainprefix` from the project config")
    }

    fn run(&self, _: &clap::ArgMatches, p: Project) -> Result<(), HEMTTError> {
        // Find Steam directory
        const ARMA3: &str = "steamapps\\common\\Arma 3";
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let binarize = hkcu.open_subkey("Software\\Valve\\Steam")?;
        let steam_path: String = binarize.get_value("SteamPath")?;
        let steam_path = PathBuf::from(steam_path);

        if !steam_path.exists() {
            return Err(HEMTTError::simple("Steam Folder from registry does not exists"));
        }

        // Check root install
        let mut root = steam_path.clone();
        root.push(ARMA3);
        if root.exists() {
            FilePatching::create_link(root, &p)?;
            Ok(())
        } else {
            let mut library_folders = steam_path;
            library_folders.push("steamapps");
            library_folders.push("libraryfolders.vdf");
            if !library_folders.exists() {
                return Err(HEMTTError::simple("No library folders, Arma 3 is probably not installed"));
            }
            let mut folders = String::new();
            open_file!(library_folders)?.read_to_string(&mut folders)?;
            let re = Regex::new(r#"(?m)"(\d)"\s+?"(.+?)""#).unwrap();
            let result = re.captures_iter(&folders);
            for cap in result {
                let mut folder = PathBuf::from(cap.get(2).unwrap().as_str());
                folder.push(ARMA3);
                if folder.exists() {
                    FilePatching::create_link(folder, &p)?;
                    return Ok(());
                }
            }
            Err(HEMTTError::simple(
                "No library folder contained Arma 3, it is probably not installed",
            ))
        }
    }
}
