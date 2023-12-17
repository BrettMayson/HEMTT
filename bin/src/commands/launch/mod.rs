mod error;

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use clap::{ArgMatches, Command};
use hemtt_common::{
    arma::dlc::DLC,
    project::{hemtt::LaunchOptions, ProjectConfig},
};
use regex::Regex;
use steamlocate::SteamDir;

use crate::{
    commands::launch::error::{
        bcle1_preset_not_found::PresetNotFound, bcle2_workshop_not_found::WorkshopNotFound,
        bcle3_workshop_mod_not_found::WorkshopModNotFound, bcle4_arma_not_found::ArmaNotFound,
        bcle5_missing_main_prefix::MissingMainPrefix,
        bcle6_launch_config_not_found::LaunchConfigNotFound,
    },
    error::Error,
    link::create_link,
    report::Report,
};

use super::dev;

#[must_use]
pub fn cli() -> Command {
    dev::add_args(
        Command::new("launch")
            .about("Test your project")
            .long_about("Builds your project in dev mode and launches Arma 3 with file patching enabled, loading your mod and any workshop mods.")
            .arg(
                clap::Arg::new("config")
                    .default_value("default")
                    .help("Launches with the specified `[hemtt.launch.<config>]` configuration"),
            )
            .arg(
                clap::Arg::new("executable")
                    .short('e')
                    .help("Executable to launch, defaults to `arma3_x64.exe`"),
            )
            .arg(
                clap::Arg::new("passthrough")
                    .raw(true)
                    .help("Passthrough additional arguments to Arma 3"),
            )
    )
}

#[allow(clippy::too_many_lines)]
/// Execute the launch command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// Will panic if the regex can not be compiled, which should never be the case in a released version
pub fn execute(matches: &ArgMatches) -> Result<Report, Error> {
    let config = ProjectConfig::from_file(&Path::new(".hemtt").join("project.toml"))?;
    let mut report = Report::new();
    let Some(mainprefix) = config.mainprefix() else {
        report.error(MissingMainPrefix::code());
        return Ok(report);
    };

    let launch_config = matches
        .get_one::<String>("config")
        .map_or_else(|| String::from("default"), std::string::ToString::to_string);
    let Some(launch) = config
        .hemtt()
        .launch(&launch_config)
        .or(if launch_config == "default" {
            Some(Cow::Owned(LaunchOptions::default()))
        } else {
            None
        })
    else {
        report.error(LaunchConfigNotFound::code(
            launch_config,
            &config.hemtt().launch_keys(),
        ));
        return Ok(report);
    };

    trace!("launch config: {:?}", launch);

    let Some(arma3dir) =
        SteamDir::locate().and_then(|mut s| s.app(&107_410).map(std::borrow::ToOwned::to_owned))
    else {
        report.error(ArmaNotFound::code());
        return Ok(report);
    };

    debug!("Arma 3 found at: {}", arma3dir.path.display());

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
        let regex = Regex::new(r"publishedid\s*=\s*(\d+);").unwrap();
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
            report.error(PresetNotFound::code(
                &launch_config,
                preset.to_string(),
                &presets,
            ));
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
        let Some(common) = arma3dir.path.parent() else {
            report.error(WorkshopNotFound::code());
            return Ok(report);
        };
        let Some(root) = common.parent() else {
            report.error(WorkshopNotFound::code());
            return Ok(report);
        };
        let workshop_folder = root.join("workshop").join("content").join("107410");
        if !workshop_folder.exists() {
            report.error(WorkshopNotFound::code());
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
                report.error(WorkshopModNotFound::code(load_mod));
            };
            mods.push(mod_path.display().to_string());
        }
    }
    if report.failed() {
        return Ok(report);
    }

    for dlc in dlc {
        mods.push(dlc.to_mod().to_string());
    }

    let mut executor = super::dev::context(matches, launch.optionals())?;

    report.merge(executor.run()?);

    let prefix_folder = arma3dir.path.join(mainprefix);
    if !prefix_folder.exists() {
        std::fs::create_dir_all(&prefix_folder)?;
    }

    let link = prefix_folder.join(executor.ctx().config().prefix());
    if !link.exists() {
        create_link(&link, executor.ctx().build_folder())?;
    }

    let mut args: Vec<String> = [
        "-skipIntro",
        "-noSplash",
        "-showScriptErrors",
        "-debug",
        "-filePatching",
    ]
    .iter()
    .map(std::string::ToString::to_string)
    .collect();
    args.append(&mut launch.parameters().to_vec());
    args.append(
        &mut matches
            .get_raw("passthrough")
            .unwrap_or_default()
            .map(|s| s.to_string_lossy().to_string())
            .collect::<Vec<_>>(),
    );
    args.push(
        mods.iter()
            .map(|s| format!("-mod=\"{s}\""))
            .collect::<Vec<_>>()
            .join(" "),
    );

    if cfg!(target_os = "windows") {
        info!(
            "Launching {:?} with:\n  {}",
            arma3dir.path.display(),
            args.join("\n  ")
        );
        std::process::Command::new({
            let mut path = arma3dir.path;
            if let Some(exe) = matches.get_one::<String>("executable") {
                let exe = PathBuf::from(exe);
                if exe.is_absolute() {
                    path = exe;
                } else {
                    path.push(exe);
                }
                if cfg!(windows) {
                    path.set_extension("exe");
                }
            } else {
                path.push(launch.executable());
            }
            path.display().to_string()
        })
        .args(args)
        .spawn()?;
    } else {
        linux_launch(&arma3dir.path, &launch.executable(), &args)?;
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
    .unwrap();
    for id in mod_regex.captures_iter(html).map(|c| c[1].to_string()) {
        if workshop.contains(&id) {
            trace!("Skipping mod {} in preset {}", id, name);
        } else {
            trace!("Found new mod {} in preset {}", id, name);
            workshop.push(id);
        }
    }
    let dlc_regex =
        Regex::new(r#"(?m)href="https?:\/\/store\.steampowered\.com\/app\/(\d+)""#).unwrap();
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

fn linux_launch(arma3dir: &Path, executable: &str, args: &[String]) -> Result<(), Error> {
    // check if flatpak steam is installed
    let flatpak = std::process::Command::new("flatpak")
        .arg("list")
        .arg("--app")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("com.valvesoftware.Steam"))?;
    if flatpak {
        warn!("A flatpak override will be created to grant access to the .hemttout directory");
        info!("Using flatpak steam with:\n  {}", args.join("\n  "));
        trace!("using flatpak override to grant access to the mod");
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
        std::process::Command::new(arma3dir.join(executable))
            .args(args)
            .spawn()?;
    }
    Ok(())
}
