use serde::{Deserialize, Serialize};

use crate::arma::dlc::DLC;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Configuration for `hemtt launch`
pub struct LaunchOptions {
    /// Workshop mods that should be launched with the mod
    workshop: Vec<String>,

    /// DLCs that should be launched with the mod
    dlc: Vec<DLC>,

    /// HTML presets that should be launched with the mod
    presets: Vec<String>,

    /// Optional addons that should be built into the mod
    optionals: Vec<String>,

    /// Mission to launch directly into the editor with
    mission: Option<String>,

    /// Extra launch parameters
    parameters: Vec<String>,

    /// Binary to launch
    executable: Option<String>,
}

impl LaunchOptions {
    #[must_use]
    /// Workshop mods that should be launched with the mod
    pub fn workshop(&self) -> &[String] {
        &self.workshop
    }

    #[must_use]
    /// DLCs that should be launched with the mod
    pub fn dlc(&self) -> &[DLC] {
        &self.dlc
    }

    #[must_use]
    /// HTML presets that should be launched with the mod
    pub fn presets(&self) -> &[String] {
        &self.presets
    }

    #[must_use]
    /// Optional addons that should be built into the mod
    pub fn optionals(&self) -> &[String] {
        &self.optionals
    }

    #[must_use]
    /// Mission to launch directly into the editor with
    pub const fn mission(&self) -> Option<&String> {
        self.mission.as_ref()
    }

    #[must_use]
    /// Extra launch parameters
    pub fn parameters(&self) -> &[String] {
        &self.parameters
    }

    #[must_use]
    /// Binary to launch, `.exe` is appended on Windows  
    /// Defaults to `arma3_x64`
    pub fn executable(&self) -> String {
        let executable = &self
            .executable
            .as_ref()
            .map_or_else(|| "arma3_x64", |e| e.as_str());
        if cfg!(target_os = "windows") {
            format!("{executable}.exe")
        } else {
            (*executable).to_string()
        }
    }

    #[must_use]
    /// Overlay two launch options  
    /// Other will take precedence
    pub fn overlay(self, other: Self) -> Self {
        let mut base = self;
        base.workshop.extend(other.workshop);
        base.dlc.extend(other.dlc);
        base.presets.extend(other.presets);
        base.optionals.extend(other.optionals);
        base.parameters.extend(other.parameters);
        if let Some(executable) = other.executable {
            base.executable = Some(executable);
        }
        if let Some(mission) = other.mission {
            base.mission = Some(mission);
        }
        base
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
/// Launch specific configuration
pub struct LaunchOptionsFile {
    #[serde(default)]
    pub(crate) extends: Option<String>,

    #[serde(default)]
    workshop: Vec<String>,

    #[serde(default)]
    dlc: Vec<DLC>,

    #[serde(default)]
    presets: Vec<String>,

    #[serde(default)]
    optionals: Vec<String>,

    #[serde(default)]
    mission: Option<String>,

    #[serde(default)]
    parameters: Vec<String>,

    #[serde(default)]
    executable: Option<String>,
}

impl LaunchOptionsFile {
    pub fn overlay(self, other: Self) -> Self {
        let mut other = other;
        other.workshop.extend(self.workshop);
        other.dlc.extend(self.dlc);
        other.presets.extend(self.presets);
        other.optionals.extend(self.optionals);
        other.parameters.extend(self.parameters);
        if let Some(executable) = self.executable {
            other.executable = Some(executable);
        }
        if let Some(mission) = self.mission {
            other.mission = Some(mission);
        }
        other
    }

    pub fn dedup(&mut self) {
        self.workshop.sort();
        self.workshop.dedup();
        self.dlc.sort();
        self.dlc.dedup();
        self.presets.sort();
        self.presets.dedup();
        self.optionals.sort();
        self.optionals.dedup();
        self.parameters.sort();
        self.parameters.dedup();
    }
}

impl From<LaunchOptionsFile> for LaunchOptions {
    fn from(file: LaunchOptionsFile) -> Self {
        Self {
            workshop: file.workshop,
            dlc: file.dlc,
            presets: file.presets,
            optionals: file.optionals,
            mission: file.mission,
            parameters: file.parameters,
            executable: file.executable,
        }
    }
}
