use serde::{Deserialize, Serialize};

use crate::arma::dlc::DLC;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Configuration for `hemtt launch`
pub struct LaunchOptions {
    /// Workshop mods that should be launched with the mod
    pub(crate) workshop: Vec<String>,

    /// DLCs that should be launched with the mod
    pub(crate) dlc: Vec<DLC>,

    /// HTML presets that should be launched with the mod
    pub(crate) presets: Vec<String>,

    /// Optional addons that should be built into the mod
    pub(crate) optionals: Vec<String>,

    /// Mission to launch directly into the editor with
    pub(crate) mission: Option<String>,

    /// Mission to launch with `hemtt launch`, used for development purposes only
    pub(crate) dev_mission: Option<String>,

    /// Extra launch parameters
    pub(crate) parameters: Vec<String>,

    /// Binary to launch
    pub(crate) executable: Option<String>,

    // Should HEMTT run binarize
    pub(crate) binarize: Option<bool>,

    // Should HEMTT use file-patching
    pub(crate) file_patching: Option<bool>,

    // Should HEMTT use multiple instances
    pub(crate) instances: Option<u8>,

    // Should HEMTT rapify
    pub(crate) rapify: Option<bool>,
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
    pub fn mission(&self) -> Option<&str> {
        self.mission.as_deref()
    }

    #[must_use]
    /// Mission to launch with `hemtt launch`, used for development purposes only
    pub fn dev_mission(&self) -> Option<&str> {
        self.dev_mission.as_deref()
    }

    pub fn set_mission(&mut self, mission: Option<String>) {
        self.mission = mission;
    }

    #[must_use]
    /// Extra launch parameters
    pub fn parameters(&self) -> &[String] {
        &self.parameters
    }

    #[must_use]
    /// Binary to launch, with `.exe`
    /// Defaults to `arma3_x64.exe`
    pub fn executable(&self) -> String {
        let executable = &self
            .executable
            .as_ref()
            .map_or_else(|| "arma3_x64", |e| e.as_str());
        if std::path::Path::new(executable)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
        {
            (*executable).to_string()
        } else {
            format!("{executable}.exe")
        }
    }

    #[must_use]
    /// Should HEMTT run binarize
    /// Defaults to `false`
    pub fn binarize(&self) -> bool {
        self.binarize.unwrap_or(false)
    }

    #[must_use]
    /// Should HEMTT use file-patching
    /// Defaults to `true`
    pub fn file_patching(&self) -> bool {
        self.file_patching.unwrap_or(true)
    }

    #[must_use]
    /// Should HEMTT use multiple instances
    /// Defaults to `1`
    pub fn instances(&self) -> u8 {
        self.instances.unwrap_or(1)
    }

    #[must_use]
    /// Should HEMTT rapify
    /// Defaults to `true`
    pub fn rapify(&self) -> bool {
        self.rapify.unwrap_or(true)
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
        if let Some(dev_mission) = other.dev_mission {
            base.dev_mission = Some(dev_mission);
        }
        if let Some(binarize) = other.binarize {
            base.binarize = Some(binarize);
        }
        if let Some(file_patching) = other.file_patching {
            base.file_patching = Some(file_patching);
        }
        if let Some(instances) = other.instances {
            base.instances = Some(instances);
        }
        if let Some(rapify) = other.rapify {
            base.rapify = Some(rapify);
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
    pub(crate) mission: Option<String>,

    #[serde(default)]
    pub(crate) dev_mission: Option<String>,

    #[serde(default)]
    parameters: Vec<String>,

    #[serde(default)]
    executable: Option<String>,

    #[serde(default)]
    binarize: Option<bool>,

    #[serde(default)]
    file_patching: Option<bool>,

    #[serde(default)]
    instances: Option<u8>,

    #[serde(default)]
    rapify: Option<bool>,
}

impl LaunchOptionsFile {
    pub fn extend(self, other: Self) -> Self {
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
        if let Some(dev_mission) = self.dev_mission {
            other.dev_mission = Some(dev_mission);
        }
        if let Some(binarize) = self.binarize {
            other.binarize = Some(binarize);
        }
        if let Some(file_patching) = self.file_patching {
            other.file_patching = Some(file_patching);
        }
        if let Some(instances) = self.instances {
            other.instances = Some(instances);
        }
        if let Some(rapify) = self.rapify {
            other.rapify = Some(rapify);
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
            dev_mission: file.dev_mission,
            parameters: file.parameters,
            executable: file.executable,
            binarize: file.binarize,
            file_patching: file.file_patching,
            instances: file.instances,
            rapify: file.rapify,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fully_defined() {
        let toml = r#"
workshop = ["123456"]
dlc = ["contact"]
presets = ["test"]
optionals = ["test"]
mission = "test"
parameters = ["test"]
executable = "test"
binarize = true
file_patching = false
instances = 2
rapify = false
"#;
        let file: LaunchOptionsFile = toml::from_str(toml).expect("failed to deserialize");
        let config = LaunchOptions::from(file);
        assert_eq!(config.workshop(), &["123456"]);
        assert_eq!(config.dlc(), &[DLC::Contact]);
        assert_eq!(config.presets(), &["test"]);
        assert_eq!(config.optionals(), &["test"]);
        assert_eq!(config.mission(), Some("test"));
        assert_eq!(config.parameters(), &["test"]);
        assert_eq!(config.executable(), "test.exe");
        assert!(config.binarize());
        assert!(!config.file_patching());
        assert_eq!(config.instances(), 2);
        assert!(!config.rapify());
    }

    #[test]
    fn default() {
        let toml = "";
        let file: LaunchOptionsFile = toml::from_str(toml).expect("failed to deserialize");
        let config = LaunchOptions::from(file);
        assert!(config.workshop().is_empty());
        assert!(config.dlc().is_empty());
        assert!(config.presets().is_empty());
        assert!(config.optionals().is_empty());
        assert!(config.mission().is_none());
        assert!(config.parameters().is_empty());
        assert_eq!(config.executable(), "arma3_x64.exe");
        assert!(!config.binarize());
        assert!(config.file_patching());
        assert_eq!(config.instances(), 1);
        assert!(config.rapify());
    }
}
