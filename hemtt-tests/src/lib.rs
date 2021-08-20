mod coverage;
use std::{io::Read, path::PathBuf};

pub use coverage::inject;
use hemtt::HEMTTError;
use hemtt_macros::open_file;
use regex::Regex;
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

pub const ARMA_STARTUP: &[&'static str] = &["-window", "-noSplash", "-skipIntro", "-name=\"hemtt_tests\"", "-noPause", "-showScriptErrors", "-debug", "-mod=\"F:\\SteamLibrary\\steamapps\\common\\Arma 3\\!Workshop\\@CBA_A3;P:\\arma\\hemtt-exp\\hemtt-tests\\src\\mod\"", r#"-init=playMission['','\hemtt\tests\addons\main\missions\sp_blufor_rifleman.Stratis',true]"#];

pub fn arma_path() -> Result<PathBuf, HEMTTError> {
    const ARMA3: &str = "steamapps\\common\\Arma 3\\arma3_x64.exe";
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let binarize = hkcu.open_subkey("Software\\Valve\\Steam")?;
    let steam_path: String = binarize.get_value("SteamPath")?;
    let steam_path = PathBuf::from(steam_path);

    if !steam_path.exists() {
        return Err(HEMTTError::Generic(
            "Steam Folder from registry does not exists".to_string(),
        ));
    }

    // Check root install
    let mut root = steam_path.clone();
    root.push(ARMA3);
    if root.exists() {
        Ok(root)
    } else {
        let mut library_folders = steam_path;
        library_folders.push("steamapps");
        library_folders.push("libraryfolders.vdf");
        if !library_folders.exists() {
            return Err(HEMTTError::Generic(
                "No library folders, Arma 3 is probably not installed".to_string(),
            ));
        }
        let mut folders = String::new();
        open_file!(library_folders)?.read_to_string(&mut folders)?;

        // Older library format
        let re = Regex::new(r#"(?m)"\d"\s+?"(.+?)""#).unwrap();
        let result = re.captures_iter(&folders);
        for cap in result {
            let mut folder = PathBuf::from(cap.get(1).unwrap().as_str());
            folder.push(ARMA3);
            if folder.exists() {
                return Ok(folder);
            }
        }

        // New library format
        let re = Regex::new(r#"(?m)"path"\s+?"(.+?)""#).unwrap();
        let result = re.captures_iter(&folders);
        for cap in result {
            let mut folder = PathBuf::from(cap.get(1).unwrap().as_str());
            folder.push(ARMA3);
            if folder.exists() {
                return Ok(folder);
            }
        }
        Err(HEMTTError::Generic(
            "No library folder contained Arma 3, it is probably not installed".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_arma_path() {
        let path = super::arma_path().unwrap();
        println!("Path: {:?}", path);
        assert_eq!(path.exists(), true);
    }
}
