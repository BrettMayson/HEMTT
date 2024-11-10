mod error;

use std::path::{Path, PathBuf};

use hemtt_common::{
    arma::dlc::DLC,
    config::{LaunchOptions, ProjectConfig},
    steam,
};
use regex::Regex;

use crate::{
    commands::launch::error::{
        bcle1_preset_not_found::PresetNotFound, bcle2_workshop_not_found::WorkshopNotFound,
        bcle3_workshop_mod_not_found::WorkshopModNotFound, bcle4_arma_not_found::ArmaNotFound,
        bcle5_missing_main_prefix::MissingMainPrefix,
        bcle6_launch_config_not_found::LaunchConfigNotFound,
        bcle7_can_not_quicklaunch::CanNotQuickLaunch, bcle8_mission_not_found::MissionNotFound,
        bcle9_mission_absolute::MissionAbsolutePath,
    },
    error::Error,
    link::create_link,
    report::Report,
};

#[derive(clap::Parser)]
/// Test your project
///
/// Builds your project in dev mode and launches Arma 3 with
/// file patching enabled, loading your mod and any workshop mods.
pub struct Command {
    #[clap(flatten)]
    launch: LaunchArgs,

    #[clap(flatten)]
    dev: super::dev::DevArgs,

    #[clap(flatten)]
    just: super::JustArgs,

    #[clap(flatten)]
    global: crate::GlobalArgs,
}

#[derive(clap::Args)]
#[allow(clippy::module_name_repetitions)]
pub struct LaunchArgs {
    #[arg(action = clap::ArgAction::Append)]
    /// Launches with the specified `[hemtt.launch.<config>]` configurations
    config: Option<Vec<String>>,
    #[arg(long, short)]
    /// Executable to launch, defaults to `arma3_x64.exe`
    executable: Option<String>,
    #[arg(raw = true)]
    /// Passthrough additional arguments to Arma 3
    passthrough: Option<Vec<String>>,
    #[arg(long, short)]
    /// Launches multiple instances of the game
    instances: Option<u8>,
    #[arg(long = "quick", short = 'Q')]
    /// Skips the build step, launching the last built version
    no_build: bool,
    #[arg(long = "no-filepatching", short = 'F')]
    /// Disables file patching
    no_filepatching: bool,
}

#[allow(clippy::too_many_lines)]
/// Execute the launch command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// Will panic if the regex can not be compiled, which should never be the case in a released version
pub fn execute(cmd: &Command) -> Result<Report, Error> {
    let config = ProjectConfig::from_file(&Path::new(".hemtt").join("project.toml"))?;
    let mut report = Report::new();
    let Some(mainprefix) = config.mainprefix() else {
        report.push(MissingMainPrefix::code());
        return Ok(report);
    };

    let configs = cmd.launch.config.clone().unwrap_or_default();

    let launch = if configs.is_empty() {
        config
            .hemtt()
            .launch()
            .get("default")
            .cloned()
            .unwrap_or_default()
    } else if let Some(launch) = configs
        .into_iter()
        .map(|c| {
            config.hemtt().launch().get(&c).cloned().map_or_else(
                || {
                    report.push(LaunchConfigNotFound::code(
                        c.to_string(),
                        &config.hemtt().launch().keys().cloned().collect::<Vec<_>>(),
                    ));
                    None
                },
                Some,
            )
        })
        .collect::<Option<Vec<_>>>()
    {
        launch.into_iter().fold(
            LaunchOptions::default(),
            hemtt_common::config::LaunchOptions::overlay,
        )
    } else {
        return Ok(report);
    };

    trace!("launch config: {:?}", launch);

    let instance_count = cmd.launch.instances.unwrap_or_else(|| launch.instances());

    let Some(arma3dir) = steam::find_app(107_410) else {
        report.push(ArmaNotFound::code());
        return Ok(report);
    };

    debug!("Arma 3 found at: {}", arma3dir.display());

    let mut mods = Vec::new();

    mods.push({
        let mut path = std::env::current_dir()?;
        path.push(".hemttout/dev");
        if cfg!(target_os = "linux") {
            format!("Z:{}", path.display())
        } else {
            path.display().to_string()
        }
    });

    let mut meta = None;
    let meta_path = std::env::current_dir()?.join("meta.cpp");
    if meta_path.exists() {
        let content = std::fs::read_to_string(meta_path)?;
        let regex = Regex::new(r"publishedid\s*=\s*(\d+);").expect("meta regex compiles");
        if let Some(id) = regex.captures(&content).map(|c| c[1].to_string()) {
            meta = Some(id);
        }
    }

    let mut workshop = launch.workshop().to_vec();
    let mut dlc = launch.dlc().to_vec();

    let presets = std::env::current_dir()?.join(".hemtt/presets");
    for preset in launch.presets() {
        trace!("Loading preset: {}", preset);
        let html = presets.join(preset).with_extension("html");
        if !html.exists() {
            report.push(PresetNotFound::code(preset.to_string(), &presets));
            continue;
        }
        let html = std::fs::read_to_string(html)?;
        let (preset_mods, preset_dlc) = read_preset(preset, &html);
        for load_mod in preset_mods {
            if !workshop.contains(&load_mod) {
                workshop.push(load_mod);
            }
        }
        for load_dlc in preset_dlc {
            if !dlc.contains(&load_dlc) {
                dlc.push(load_dlc);
            }
        }
    }
    if report.failed() {
        return Ok(report);
    }

    // climb to the workshop folder
    if !workshop.is_empty() {
        let Some(common) = arma3dir.parent() else {
            report.push(WorkshopNotFound::code());
            return Ok(report);
        };
        let Some(root) = common.parent() else {
            report.push(WorkshopNotFound::code());
            return Ok(report);
        };
        let workshop_folder = root.join("workshop").join("content").join("107410");
        if !workshop_folder.exists() {
            report.push(WorkshopNotFound::code());
            return Ok(report);
        };
        for load_mod in workshop {
            if Some(load_mod.clone()) == meta {
                warn!(
                    "Skipping mod {} as it is the same as the project's meta.cpp id",
                    load_mod
                );
                continue;
            }
            let mod_path = workshop_folder.join(&load_mod);
            if !mod_path.exists() {
                report.push(WorkshopModNotFound::code(load_mod));
            };
            if cfg!(windows) {
                mods.push(mod_path.display().to_string());
            } else {
                mods.push(format!("Z:{}", mod_path.display()));
            }
        }
    }
    if report.failed() {
        return Ok(report);
    }

    for dlc in dlc {
        mods.push(dlc.to_mod().to_string());
    }

    let mut args: Vec<String> = ["-skipIntro", "-noSplash", "-showScriptErrors", "-debug"]
        .iter()
        .map(std::string::ToString::to_string)
        .collect();
    args.append(&mut launch.parameters().to_vec());
    args.append(&mut cmd.launch.passthrough.clone().unwrap_or_default());
    args.push(
        mods.iter()
            .map(|s| format!("-mod=\"{s}\""))
            .collect::<Vec<_>>()
            .join(" "),
    );

    if let Some(mission) = launch.mission() {
        let mut path = PathBuf::from(mission);

        if path.is_absolute() {
            report.push(MissionAbsolutePath::code(mission.to_string()));
            return Ok(report);
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
            return Ok(report);
        }
    }

    if cmd.launch.no_build {
        warn!("Using Quick Launch! HEMTT will not rebuild the project");
        if !std::env::current_dir()?.join(".hemttout/dev").exists() {
            report.push(CanNotQuickLaunch::code(
                "no dev build found in .hemttout/dev".to_string(),
            ));
            return Ok(report);
        }

        let prefix_folder = arma3dir.join(mainprefix);
        let link = prefix_folder.join(config.prefix());
        if !prefix_folder.exists() || !link.exists() {
            report.push(CanNotQuickLaunch::code(
                "link does not exist in the Arma 3 folder".to_string(),
            ));
            return Ok(report);
        }
    } else {
        let mut executor = super::dev::context(
            &cmd.dev,
            &cmd.just,
            launch.optionals(),
            launch.binarize(),
            launch.rapify(),
        )?;

        report.merge(executor.run()?);

        if report.failed() {
            return Ok(report);
        }

        let prefix_folder = arma3dir.join(mainprefix);
        if !prefix_folder.exists() {
            std::fs::create_dir_all(&prefix_folder)?;
        }

        let link = prefix_folder.join(executor.ctx().config().prefix());
        if !link.exists() {
            create_link(
                &link,
                executor.ctx().build_folder().expect("build folder exists"),
            )?;
        }
    }

    // let with_server = matches.get_flag("server");
    let with_server = false;

    let mut instances = vec![];
    if with_server {
        let mut args = args.clone();
        args.push("-server".to_string());
        instances.push(args);
    }
    for _ in 0..instance_count {
        let mut args = args.clone();
        if with_server {
            args.push("-connect=127.0.0.1".to_string());
        } else if launch.file_patching() && !cmd.launch.no_filepatching {
            args.push("-filePatching".to_string());
        }
        instances.push(args);
    }

    if cfg!(target_os = "windows") {
        let mut path = arma3dir.clone();
        if let Some(exe) = &cmd.launch.executable {
            let exe = PathBuf::from(exe);
            if exe.is_absolute() {
                path = exe;
            } else {
                path.push(exe);
            }
            path.set_extension("exe");
        } else {
            path.push(launch.executable());
        }
        for instance in instances {
            windows_launch(&arma3dir, &path, &instance)?;
        }
    } else {
        if launch.executable() != "arma3_x64.exe" {
            warn!("Currently, only Windows supports specifying the executable");
        }
        for instance in instances {
            linux_launch(&instance)?;
        }
    }

    Ok(report)
}

/// Read a preset file and return the mods and DLCs
///
/// # Panics
/// Will panic if the regex can not be compiled, which should never be the case in a released version
pub fn read_preset(name: &str, html: &str) -> (Vec<String>, Vec<DLC>) {
    let mut workshop = Vec::new();
    let mut dlc = Vec::new();
    let mod_regex = Regex::new(
        r#"(?m)href="https?:\/\/steamcommunity\.com\/sharedfiles\/filedetails\/\?id=(\d+)""#,
    )
    .expect("mod regex compiles");
    for id in mod_regex.captures_iter(html).map(|c| c[1].to_string()) {
        if workshop.contains(&id) {
            trace!("Skipping mod {} in preset {}", id, name);
        } else {
            trace!("Found new mod {} in preset {}", id, name);
            workshop.push(id);
        }
    }
    let dlc_regex = Regex::new(r#"(?m)href="https?:\/\/store\.steampowered\.com\/app\/(\d+)""#)
        .expect("dlc regex compiles");
    for id in dlc_regex.captures_iter(html).map(|c| c[1].to_string()) {
        let Ok(preset_dlc) = DLC::try_from(id.clone()) else {
            warn!(
                "Preset {} requires DLC {}, but HEMTT does not recognize it",
                name, id
            );
            continue;
        };
        if dlc.contains(&preset_dlc) {
            trace!("Skipping DLC {} in preset {}", id, name);
        } else {
            trace!("Found new DLC {} in preset {}", id, name);
            dlc.push(preset_dlc);
        }
    }
    (workshop, dlc)
}

fn windows_launch(arma3dir: &Path, executable: &PathBuf, args: &[String]) -> Result<(), Error> {
    info!(
        "Launching {:?} with:\n  {}",
        arma3dir.display(),
        args.join("\n  ")
    );
    std::process::Command::new(executable).args(args).spawn()?;
    Ok(())
}

fn linux_launch(args: &[String]) -> Result<(), Error> {
    // check if flatpak steam is installed
    let flatpak = std::process::Command::new("flatpak")
        .arg("list")
        .arg("--app")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("com.valvesoftware.Steam"))?;
    if flatpak {
        warn!(
            "A flatpak override will be created to grant Steam access to the .hemttout directory"
        );
        info!("Using flatpak steam with:\n  {}", args.join("\n  "));
        std::process::Command::new("flatpak")
            .arg("override")
            .arg("--user")
            .arg("com.valvesoftware.Steam")
            .arg(format!("--filesystem={}", {
                let mut path = std::env::current_dir()?;
                path.push(".hemttout/dev");
                path.display().to_string()
            }))
            .spawn()?
            .wait()?;
        std::process::Command::new("flatpak")
            .arg("run")
            .arg("com.valvesoftware.Steam")
            .arg("-applaunch")
            .arg("107410")
            .arg("-nolauncher")
            .args(args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
    } else {
        info!("Using native steam with:\n  {}", args.join("\n  "));
        std::process::Command::new("steam")
            .arg("-applaunch")
            .arg("107410")
            .arg("-nolauncher")
            .args(args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
    }
    Ok(())
}
