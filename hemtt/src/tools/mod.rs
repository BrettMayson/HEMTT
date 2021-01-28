use std::path::PathBuf;

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
pub fn find_bi_tool(_tool: &str) -> Result<PathBuf, HEMTTError> {
    unreachable!();
}
