use std::{path::PathBuf, process::Command, sync::OnceLock};

use hemtt_common::steam;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BiTool {
    Binarize,
    Arma3,
}

static COMPATIBILITY: OnceLock<CompatibilityTool> = OnceLock::new();

impl BiTool {
    #[allow(dead_code)]
    pub fn registry_key(&self) -> &str {
        match self {
            Self::Binarize => "Software\\Bohemia Interactive\\binarize",
            Self::Arma3 => unreachable!("Arma3 does not have a registry key"),
        }
    }

    #[allow(dead_code)]
    pub fn value_name(&self) -> &str {
        match self {
            Self::Binarize => "path",
            Self::Arma3 => unreachable!("Arma3 does not have a registry value"),
        }
    }

    pub fn is_installed(self) -> bool {
        if let Some(path) = Self::locate(self) {
            return path.exists();
        }
        false
    }

    pub fn locate(self) -> Option<PathBuf> {
        let path = if self == Self::Binarize
            && let Ok(path) = std::env::var("HEMTT_BINARIZE_PATH")
        {
            trace!("Using Binarize path from HEMTT_BINARIZE_PATH");
            Some(PathBuf::from(path))
        } else {
            locate(self)
        };
        let path = match self {
            Self::Binarize => path.map(|f| f.join("Binarize").join("binarize_x64.exe")),
            Self::Arma3 => path.map(|f| f.join("arma3_x64.exe")),
        };
        if path.as_ref().is_some_and(|p| p.exists()) {
            path
        } else {
            None
        }
    }

    pub fn command(self) -> Result<Command, std::io::Error> {
        let Some(exe) = self.locate() else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("{self:?} not found"),
            ));
        };
        Ok(if cfg!(windows) {
            Command::new(exe)
        } else {
            let compatibility = COMPATIBILITY.get_or_init(|| {
                CompatibilityTool::determine().unwrap_or(CompatibilityTool::Wine64)
            });
            match compatibility {
                CompatibilityTool::Wine64 | CompatibilityTool::Wine => {
                    let mut cmd = Command::new(compatibility.to_string());
                    cmd.arg(exe);
                    cmd.env("WINEPREFIX", "/tmp/hemtt-wine");
                    fs_err::create_dir_all("/tmp/hemtt-wine")
                        .expect("should be able to create wine prefix");
                    cmd
                }
                CompatibilityTool::Proton => {
                    let mut home = dirs::home_dir().expect("home directory exists");
                    if exe.display().to_string().contains("/.var/") {
                        home = home.join(".var/app/com.valvesoftware.Steam");
                    }
                    let mut cmd = Command::new({
                        home.join(
                            ".local/share/Steam/steamapps/common/SteamLinuxRuntime_sniper/run",
                        )
                    });
                    cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", 
                        home.join(".local/share/Steam")
                    ).env(
                        "STEAM_COMPAT_DATA_PATH",
                        home.join(".local/share/Steam/steamapps/compatdata/233800")
                    ).env("STEAM_COMPAT_INSTALL_PATH", "/tmp/hemtt-scip").arg("--").arg(
                        home.join(".local/share/Steam/steamapps/common/Proton - Experimental/proton")
                    ).arg("run").arg(
                        home.join(".local/share/Steam/steamapps/common/Arma 3 Tools/Binarize/binarize_x64.exe")
                    );
                    cmd
                }
            }
        })
    }
}

fn locate(tool: BiTool) -> Option<PathBuf> {
    if tool == BiTool::Arma3 {
        return steam::find_app(107_410);
    }

    if let Some(registry) = locate_registry(tool) {
        return Some(registry);
    }

    if cfg!(windows) {
        None
    } else {
        let default = dirs::home_dir()
            .expect("home directory exists")
            .join(".local/share/arma3tools");
        if let Ok(path) = std::env::var("HEMTT_BI_TOOLS") {
            Some(PathBuf::from(path))
        } else if !default.exists() {
            Some(steam::find_app(233_800)?)
        } else {
            Some(default)
        }
    }
}

#[cfg(windows)]
fn locate_registry(tool: &BiTool) -> Option<PathBuf> {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let Ok(key) = hkcu.open_subkey(tool.registry_key()) else {
        return None;
    };
    let Ok(path) = key.get_value::<String, _>(tool.value_name()) else {
        return None;
    };
    Some(PathBuf::from(path))
}

#[cfg(not(windows))]
const fn locate_registry(_: BiTool) -> Option<PathBuf> {
    None
}

#[derive(Default)]
#[allow(dead_code)]
pub enum CompatibilityTool {
    #[default]
    Wine64,
    Wine,
    Proton,
}

impl CompatibilityTool {
    #[allow(dead_code)]
    pub fn determine() -> Option<Self> {
        use dirs::home_dir;
        if cfg!(windows) {
            return None;
        }
        let mut cmd = Command::new("wine64");
        cmd.arg("--version");
        if cmd.output().is_ok() {
            return Some(Self::Wine64);
        }
        let mut cmd = Command::new("wine");
        cmd.arg("--version");
        if cmd.output().is_ok() {
            return Some(Self::Wine);
        }
        if home_dir()
            .expect("home directory exists")
            .join(".local/share/Steam/steamapps/common/SteamLinuxRuntime_sniper/run")
            .exists()
        {
            return Some(Self::Proton);
        }
        None
    }
}

impl std::fmt::Display for CompatibilityTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wine64 => write!(f, "wine64"),
            Self::Wine => write!(f, "wine"),
            Self::Proton => write!(f, "proton"),
        }
    }
}
