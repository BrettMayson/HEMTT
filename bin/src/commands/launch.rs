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

use crate::{error::Error, link::create_link};

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
pub fn execute(matches: &ArgMatches) -> Result<(), Error> {
    let config = ProjectConfig::from_file(&Path::new(".hemtt").join("project.toml"))?;
    let Some(mainprefix) = config.mainprefix() else {
        return Err(Error::MainPrefixNotFound(String::from(
            "Required for launch",
        )));
    };

    let launch_config = matches
        .get_one::<String>("config")
        .map_or_else(|| String::from("default"), std::string::ToString::to_string);
    let launch = config
        .hemtt()
        .launch(&launch_config)
        .or(if launch_config == "default" {
            Some(Cow::Owned(LaunchOptions::default()))
        } else {
            None
        })
        .ok_or(Error::LaunchConfigNotFound(launch_config.to_string()))?;

    trace!("launch config: {:?}", launch);

    let Some(arma3dir) =
        SteamDir::locate().and_then(|mut s| s.app(&107_410).map(std::borrow::ToOwned::to_owned))
    else {
        return Err(Error::Arma3NotFound);
    };

    debug!("Arma 3 found at: {}", arma3dir.path.display());

    let mut mods = Vec::new();

    mods.push({
        let mut path = std::env::current_dir()?;
        path.push(".hemttout/dev");
        path.display().to_string()
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

    for preset in launch.presets() {
        trace!("Loading preset: {}", preset);
        let html = std::env::current_dir()?
            .join(".hemtt/presets")
            .join(preset)
            .with_extension("html");
        if !html.exists() {
            return Err(Error::PresetNotFound(preset.to_string()));
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

    // climb to the workshop folder
    if !launch.workshop().is_empty() {
        let Some(common) = arma3dir.path.parent() else {
            return Err(Error::WorkshopNotFound);
        };
        let Some(root) = common.parent() else {
            return Err(Error::WorkshopNotFound);
        };
        let workshop_folder = root.join("workshop").join("content").join("107410");
        if !workshop_folder.exists() {
            return Err(Error::WorkshopNotFound);
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
                return Err(Error::WorkshopModNotFound(load_mod));
            };
            mods.push(mod_path.display().to_string());
        }
    }

    for dlc in dlc {
        mods.push(dlc.to_mod().to_string());
    }

    let ctx = super::dev::execute(matches, launch.optionals())?;

    let prefix_folder = arma3dir.path.join(mainprefix);
    if !prefix_folder.exists() {
        std::fs::create_dir_all(&prefix_folder)?;
    }

    let link = prefix_folder.join(ctx.config().prefix());
    if !link.exists() {
        create_link(&link, ctx.build_folder())?;
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
    args.push(format!("-mod=\"{}\"", mods.join(";"),));

    info!(
        "Launching {:?} with: {:?}",
        arma3dir.path.display(),
        args.join(" ")
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

    Ok(())
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
        if !workshop.contains(&id) {
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
        if !dlc.contains(&preset_dlc) {
            dlc.push(preset_dlc);
        }
    }
    (workshop, dlc)
}
