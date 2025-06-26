pub mod build;
pub mod check;
pub mod dev;
pub mod launch;
pub mod release;

use std::{collections::HashMap, path::Path, sync::Arc};

use launch::LaunchOptions;
use serde::{Deserialize, Serialize};

use crate::Error;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, PartialEq, Eq)]
/// Configure HEMTT commands
pub struct HemttConfig {
    check: check::CheckOptions,

    dev: dev::DevOptions,

    launch: HashMap<String, launch::LaunchOptions>,

    build: build::BuildOptions,

    release: release::ReleaseOptions,
}

impl HemttConfig {
    /// Get the check options
    pub const fn check(&self) -> &check::CheckOptions {
        &self.check
    }

    /// Get the dev options
    pub const fn dev(&self) -> &dev::DevOptions {
        &self.dev
    }

    /// Get the launch options
    pub const fn launch(&self) -> &HashMap<String, LaunchOptions> {
        &self.launch
    }

    /// Get the build options
    pub const fn build(&self) -> &build::BuildOptions {
        &self.build
    }

    /// Get the release options
    pub const fn release(&self) -> &release::ReleaseOptions {
        &self.release
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RuntimeArguments {
    is_release: bool,
    is_pedantic: bool,
    is_just: bool,

    explicit_lints: Arc<[String]>,
}

impl RuntimeArguments {
    #[must_use]
    pub const fn is_release(&self) -> bool {
        self.is_release
    }

    #[must_use]
    pub fn with_release(self, is_release: bool) -> Self {
        Self { is_release, ..self }
    }

    #[must_use]
    pub const fn is_pedantic(&self) -> bool {
        self.is_pedantic
    }

    #[must_use]
    pub fn with_pedantic(self, is_pedantic: bool) -> Self {
        Self {
            is_pedantic,
            ..self
        }
    }

    #[must_use]
    pub const fn is_just(&self) -> bool {
        self.is_just
    }

    #[must_use]
    pub fn with_just(self, is_just: bool) -> Self {
        Self { is_just, ..self }
    }

    #[must_use]
    pub fn explicit_lints(&self) -> &[String] {
        &self.explicit_lints
    }

    #[must_use]
    pub fn with_explicit_lints(self, explicit_lints: Vec<String>) -> Self {
        Self {
            explicit_lints: explicit_lints.into(),
            ..self
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
/// Feature specific configuration
pub struct HemttSectionFile {
    #[serde(default)]
    check: check::CheckOptionsFile,

    #[serde(default)]
    dev: dev::DevOptionsFile,

    #[serde(default)]
    launch: HashMap<String, launch::LaunchOptionsFile>,

    #[serde(default)]
    build: build::BuildOptionsFile,

    #[serde(default)]
    release: release::ReleaseOptionsFile,
}

impl HemttSectionFile {
    pub fn into_config(self, path: &Path, prefix: &str) -> Result<HemttConfig, Error> {
        let mut launch_path = path.to_path_buf();
        launch_path.set_file_name("launch.toml");
        let launch_source = if launch_path.exists() {
            if self.launch.is_empty() {
                let launch_source = std::fs::read_to_string(&launch_path)?;
                if launch_source.contains("[hemtt.launch") {
                    return Err(Error::ConfigInvalid(
                        "Configs in `launch.toml` do not need to be under `[hemtt.launch]`."
                            .to_string(),
                    ));
                }
                toml::from_str::<HashMap<String, launch::LaunchOptionsFile>>(&launch_source)?
            } else {
                return Err(Error::LaunchConfigConflict);
            }
        } else {
            self.launch
        };
        if launch_source
            .iter()
            .any(|(k, v)| k != "photoshoot" && v.dev_mission.is_some())
        {
            return Err(Error::ConfigInvalid(
                "dev_mission is only allowed in the photoshoot preset.".to_string(),
            ));
        }
        if launch_source
            .iter()
            .any(|(k, v)| k == "photoshoot" && v.mission.is_some())
        {
            return Err(Error::ConfigInvalid(
                "mission is not allowed in the photoshoot preset.".to_string(),
            ));
        }
        Ok(HemttConfig {
            check: self.check.into(),
            dev: self.dev.into(),
            launch: {
                launch_source
                    .clone()
                    .into_iter()
                    .map(|(k, v)| {
                        let mut base = v;
                        while let Some(extends) = &base.extends {
                            if extends == &k {
                                return Err(Error::LaunchConfigExtendsSelf(k));
                            }
                            if let Some(extends) = launch_source.get(extends) {
                                base = base.extend(extends.clone());
                            } else {
                                return Err(Error::LaunchConfigExtendsMissing(
                                    k,
                                    extends.to_string(),
                                ));
                            }
                        }
                        base.dedup();
                        Ok((k, base.into()))
                    })
                    .collect::<Result<_, _>>()?
            },
            build: self.build.into(),
            release: self.release.into_config(prefix),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::arma::dlc::DLC;

    use super::*;

    #[test]
    fn extends() {
        let toml = r#"
[launch.base]
workshop = ["123456"]
dlc = ["contact"]

[launch.test]
extends = "base"
mission = "test"
workshop = ["654321"]
dlc = ["spe"]
file_patching = false
"#;
        let file: HemttSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = file
            .into_config(Path::new("."), "test")
            .expect("failed to convert");
        assert_eq!(
            config
                .launch()
                .get("test")
                .expect("has test preset")
                .workshop(),
            &["123456", "654321"]
        );
        assert_eq!(
            config.launch().get("test").expect("has test preset").dlc(),
            &[DLC::Contact, DLC::Spearhead1944]
        );
        assert_eq!(
            config
                .launch()
                .get("test")
                .expect("has test preset")
                .mission(),
            Some("test")
        );
        assert!(
            !config
                .launch()
                .get("test")
                .expect("has test preset")
                .file_patching()
        );
    }

    #[test]
    fn extends_missing() {
        let toml = r#"
[launch.test]
extends = "base"
mission = "test"
"#;
        let file: HemttSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = file.into_config(Path::new("."), "test");
        assert!(config.is_err());
    }

    #[test]
    fn extends_self() {
        let toml = r#"
[launch.test]
extends = "test"
mission = "test"
"#;
        let file: HemttSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = file.into_config(Path::new("."), "test");
        assert!(config.is_err());
    }
}
