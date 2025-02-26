use std::{
    path::{Path, PathBuf},
    process::Child,
};

use hemtt_common::{arma::dlc::DLC, config::LaunchOptions, steam};
use regex::Regex;

use crate::{
    Error,
    commands::launch::{
        error::{bcle1_preset_not_found::PresetNotFound, bcle4_arma_not_found::ArmaNotFound},
        preset,
    },
    report::Report,
};

use super::{
    LaunchArgs,
    error::{
        bcle2_workshop_not_found::WorkshopNotFound,
        bcle3_workshop_mod_not_found::WorkshopModNotFound,
        bcle8_mission_not_found::MissionNotFound, bcle9_mission_absolute::MissionAbsolutePath,
    },
};

pub struct Launcher {
    executable: String,
    dlc: Vec<DLC>,
    workshop: Vec<String>,
    options: Vec<String>,
    arma3: PathBuf,
    mission: Option<String>,
    instances: u8,
    file_patching: bool,
}

impl Launcher {
    /// Creates a new launcher
    ///
    /// # Errors
    /// [`Error::Io`] if the current directory could not be determined
    pub fn new(
        launch: &LaunchArgs,
        options: &LaunchOptions,
    ) -> Result<(Report, Option<Self>), Error> {
        let mut report = Report::new();
        let Some(arma3) = steam::find_app(107_410) else {
            report.push(ArmaNotFound::code());
            return Ok((report, None));
        };
        debug!("Arma 3 found at: {}", arma3.display());
        let mut launcher = Self {
            instances: launch.instances.unwrap_or_else(|| options.instances()),
            file_patching: options.file_patching() && !launch.no_filepatching,
            executable: options.executable(),
            dlc: options.dlc().to_vec(),
            workshop: options.workshop().to_vec(),
            mission: options.mission().map(std::string::ToString::to_string),
            options: {
                let mut args = ["-skipIntro", "-noSplash", "-showScriptErrors", "-debug"]
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>();
                args.append(&mut options.parameters().to_vec());
                args.append(&mut launch.passthrough.clone().unwrap_or_default());
                args
            },
            arma3,
        };
        for preset in options.presets() {
            launcher.add_preset(preset, &mut report)?;
        }
        if report.failed() {
            return Ok((report, None));
        }
        Ok((report, Some(launcher)))
    }

    /// Adds the current project to the mod list
    ///
    /// # Errors
    /// [`Error::Io`] if the current directory could not be determined
    pub fn add_self_mod(&mut self) -> Result<(), Error> {
        self.workshop.push({
            let mut path = std::env::current_dir()?;
            path.push(".hemttout/dev");
            path.display().to_string()
        });
        Ok(())
    }

    /// Adds a preset to the mod list
    ///
    /// # Errors
    /// [`Error::Io`] if the current directory could not be determined
    /// [`Error::Io`] if the preset could not be read
    pub fn add_preset(&mut self, preset: &str, report: &mut Report) -> Result<(), Error> {
        let presets = std::env::current_dir()?.join(".hemtt/presets");
        trace!("Loading preset: {}", preset);
        let html = presets.join(preset).with_extension("html");
        if !html.exists() {
            report.push(PresetNotFound::code(preset.to_string(), &presets));
            return Ok(());
        }
        let html = std::fs::read_to_string(html)?;
        let (preset_mods, preset_dlc) = preset::read(preset, &html);
        for load_mod in preset_mods {
            if !self.workshop.contains(&load_mod) {
                self.workshop.push(load_mod);
            }
        }
        for load_dlc in preset_dlc {
            if !self.dlc.contains(&load_dlc) {
                self.dlc.push(load_dlc);
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    /// Launches the game
    ///
    /// # Errors
    /// [`Error::Io`] if the current directory could not be determined
    ///
    /// # Panics
    /// If regex fails to compile
    pub fn launch(
        &self,
        mut args: Vec<String>,
        report: &mut Report,
    ) -> Result<Option<Child>, Error> {
        let mut mods = Vec::new();
        if !self.workshop.is_empty() {
            let Some(common) = self.arma3.parent() else {
                report.push(WorkshopNotFound::code());
                return Ok(None);
            };
            let Some(root) = common.parent() else {
                report.push(WorkshopNotFound::code());
                return Ok(None);
            };
            let workshop_folder = root.join("workshop").join("content").join("107410");
            if !workshop_folder.exists() {
                report.push(WorkshopNotFound::code());
                return Ok(None);
            };

            let mut meta = None;
            let meta_path = std::env::current_dir()?.join("meta.cpp");
            if meta_path.exists() {
                let content = std::fs::read_to_string(meta_path)?;
                let regex = Regex::new(r"publishedid\s*=\s*(\d+);").expect("meta regex compiles");
                if let Some(id) = regex.captures(&content).map(|c| c[1].to_string()) {
                    meta = Some(id);
                }
            }

            for load_mod in &self.workshop {
                if Some(load_mod.clone()) == meta {
                    warn!(
                        "Skipping mod {} as it is the same as the project's meta.cpp id",
                        load_mod
                    );
                    continue;
                }
                let mod_path = workshop_folder.join(load_mod);
                if !mod_path.exists() {
                    report.push(WorkshopModNotFound::code(load_mod.to_string()));
                };
                if cfg!(windows) {
                    mods.push(mod_path.display().to_string());
                } else {
                    mods.push(format!("Z:{}", mod_path.display()));
                }
            }
        }
        if report.failed() {
            return Ok(None);
        }

        let mut dlc = self.dlc.clone();
        dlc.sort();
        dlc.dedup();
        for dlc in dlc {
            args.push(format!("-mod=\"{}\"", dlc.to_mod()));
        }
        mods.sort();
        mods.dedup();
        for m in mods {
            args.push(format!("-mod=\"{m}\""));
        }
        args.extend(self.options.clone());

        if let Some(mission) = &self.mission {
            let mut path = PathBuf::from(mission);

            if path.is_absolute() {
                report.push(MissionAbsolutePath::code(mission.to_string()));
                return Ok(None);
            }
            path = std::env::current_dir()?.join(mission);

            if !path.ends_with("mission.sqm") {
                path.push("mission.sqm");
            }

            if !path.is_file() {
                path = std::env::current_dir()?
                    .join(".hemtt")
                    .join("missions")
                    .join(mission)
                    .join("mission.sqm");
            }

            if path.is_file() {
                args.push(format!("\"{}\"", path.display()));
            } else {
                report.push(MissionNotFound::code(
                    mission.to_string(),
                    &std::env::current_dir()?,
                ));
                return Ok(None);
            }
        }

        let mut instances = Vec::new();
        for _ in 0..self.instances {
            let mut args = args.clone();
            // if with_server {
            if false {
                args.push("-connect=127.0.0.1".to_string());
            } else if self.file_patching {
                args.push("-filePatching".to_string());
            }
            instances.push(args);
        }

        if instances.len() == 1 {
            Ok(Some(if cfg!(target_os = "windows") {
                super::platforms::windows(&self.arma3, &self.executable, &instances[0])?
            } else {
                super::platforms::linux(&instances[0])?
            }))
        } else {
            let mut children = Vec::new();
            for instance in instances {
                children.push(if cfg!(target_os = "windows") {
                    super::platforms::windows(&self.arma3, &self.executable, &instance)?
                } else {
                    super::platforms::linux(&instance)?
                });
            }
            Ok(Some(
                children.into_iter().next().expect("At least one child"),
            ))
        }
    }

    #[must_use]
    pub fn arma3dir(&self) -> &Path {
        &self.arma3
    }

    pub fn add_dlcs(&mut self, dlcs: Vec<DLC>) {
        self.dlc.extend(dlcs);
    }

    #[must_use]
    pub fn with_dlcs(mut self, dlcs: Vec<DLC>) -> Self {
        self.add_dlcs(dlcs);
        self
    }

    #[must_use]
    pub const fn dlcs(&self) -> &Vec<DLC> {
        &self.dlc
    }

    pub fn add_mods(&mut self, mods: Vec<Mod>) -> Report {
        let mut report = Report::new();
        let Some(common) = self.arma3.parent() else {
            report.push(WorkshopNotFound::code());
            return report;
        };
        let Some(root) = common.parent() else {
            report.push(WorkshopNotFound::code());
            return report;
        };
        let workshop_folder = root.join("workshop").join("content").join("107410");
        if !workshop_folder.exists() {
            report.push(WorkshopNotFound::code());
            return report;
        };
        for m in mods {
            self.workshop.push(match m {
                Mod::Workshop(id) => {
                    let workshop = workshop_folder.join(&id);
                    if !workshop.exists() {
                        report.push(WorkshopNotFound::code());
                    }
                    workshop.display().to_string()
                }
                Mod::Local(path) => path,
            });
        }
        report
    }

    #[must_use]
    pub fn with_mods(mut self, mods: Vec<Mod>) -> Self {
        self.add_mods(mods);
        self
    }

    pub fn add_external_mod(&mut self, path: String) {
        self.workshop.push(path);
    }

    pub fn add_options(&mut self, options: Vec<String>) {
        self.options.extend(options);
    }

    #[must_use]
    pub fn with_options(mut self, options: Vec<String>) -> Self {
        self.add_options(options);
        self
    }
}

pub enum Mod {
    Workshop(String),
    Local(String),
}

impl Mod {
    #[must_use]
    pub fn path(&self, workshop: &Path) -> String {
        match self {
            Self::Workshop(id) => workshop.join(id).display().to_string(),
            Self::Local(path) => path.clone(),
        }
    }
}
