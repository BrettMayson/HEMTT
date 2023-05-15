use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Features {
    #[serde(default)]
    config: hemtt_config::Options,

    #[serde(default)]
    dev: DevOptions,

    #[serde(default)]
    launch: LaunchOptions,

    #[serde(default)]
    build: BuildOptions,

    #[serde(default)]
    release: ReleaseOptions,
}

impl Features {
    #[must_use]
    pub const fn config(&self) -> &hemtt_config::Options {
        &self.config
    }

    #[must_use]
    pub const fn dev(&self) -> &DevOptions {
        &self.dev
    }

    #[must_use]
    pub const fn launch(&self) -> &LaunchOptions {
        &self.launch
    }

    #[must_use]
    pub const fn build(&self) -> &BuildOptions {
        &self.build
    }

    #[must_use]
    pub const fn release(&self) -> &ReleaseOptions {
        &self.release
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DevOptions {
    #[serde(default)]
    exclude: Vec<String>,
}

impl DevOptions {
    #[must_use]
    pub fn exclude(&self) -> &[String] {
        &self.exclude
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LaunchOptions {
    #[serde(default)]
    /// Workshop mods that should be launched with the mod
    workshop: Vec<String>,

    #[serde(default)]
    /// Extra launch parameters
    parameters: Vec<String>,

    #[serde(default)]
    /// Binary to launch, defaults to `arma3_x64.exe`
    executable: Option<String>,
}

impl LaunchOptions {
    #[must_use]
    pub fn workshop(&self) -> &[String] {
        &self.workshop
    }

    #[must_use]
    pub fn parameters(&self) -> &[String] {
        &self.parameters
    }

    #[must_use]
    pub fn executable(&self) -> String {
        let executable = self
            .executable
            .clone()
            .unwrap_or_else(|| "arma3_x64".to_owned());
        if cfg!(target_os = "windows") {
            format!("{executable}.exe")
        } else {
            executable
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BuildOptions {
    #[serde(default)]
    /// Should optionals be built into their own mod?
    /// Default: true
    optional_mod_folders: Option<bool>,
}

impl BuildOptions {
    #[must_use]
    pub const fn optional_mod_folders(&self) -> bool {
        if let Some(optional) = self.optional_mod_folders {
            optional
        } else {
            true
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReleaseOptions {
    #[serde(default)]
    /// The folder name of the project
    /// Default: `prefix`
    folder: Option<String>,
    #[serde(default)]
    /// Should the PBOs be signed?
    /// Default: true
    sign: Option<bool>,
    #[serde(default)]
    /// Create an archive of the release
    /// Default: true
    archive: Option<bool>,
}

impl ReleaseOptions {
    #[must_use]
    pub fn folder(&self) -> Option<String> {
        self.folder.clone()
    }

    #[must_use]
    pub const fn sign(&self) -> bool {
        if let Some(sign) = self.sign {
            sign
        } else {
            true
        }
    }

    #[must_use]
    pub const fn archive(&self) -> bool {
        if let Some(archive) = self.archive {
            archive
        } else {
            true
        }
    }
}
