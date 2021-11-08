use std::path::PathBuf;

#[cfg(windows)]
use crate as hemtt;
#[cfg(windows)]
use hemtt_macros::open_file;

use crate::HEMTTError;

#[cfg(windows)]
/// Locate a BI tool
///
/// Arguments:
/// * `tool`: Name of the BI tool
///
/// ```
/// let bin_exe = hemtt::tools::find_bi_tool("binarize");
/// ```
pub fn find_bi_tool(tool: &str) -> Result<PathBuf, HEMTTError> {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let binarize = hkcu.open_subkey(format!("Software\\Bohemia Interactive\\{}", tool))?;
    let value: String = binarize.get_value("path")?;

    Ok(PathBuf::from(value).join(format!("{}_x64.exe", tool)))
}

#[cfg(not(windows))]
pub const fn find_bi_tool(_tool: &str) -> Result<PathBuf, HEMTTError> {
    unreachable!();
}

#[cfg(windows)]
pub fn find_arma_path() -> Result<PathBuf, HEMTTError> {
    use std::io::Read;

    use regex::Regex;

    const ARMA3: &str = "steamapps\\common\\Arma 3\\arma3_x64.exe";
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
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

#[cfg(not(windows))]
pub const fn find_arma_path() -> Result<PathBuf, HEMTTError> {
    unreachable!();
}
