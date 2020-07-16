use std::path::PathBuf;

use crate::HEMTTError;

#[cfg(windows)]
/// Locate a BI tool on Windows
///
/// Arguments:
/// * `tool`: Name of the BI tool
///
/// ```rs
/// let bin_exe = find_exe("binarize")?;
/// ```
pub fn find_exe(tool: &str) -> Result<PathBuf, HEMTTError> {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let binarize = hkcu.open_subkey(format!("Software\\Bohemia Interactive\\{}", tool))?;
    let value: String = binarize.get_value("path")?;

    Ok(PathBuf::from(value).join(format!("{}_x64.exe", tool)))
}

#[cfg(not(windows))]
pub fn find_binarize_exe() -> Result<PathBuf, ArmakeError> {
    unreachable!();
}
