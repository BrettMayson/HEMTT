use std::path::{Path, PathBuf};

use clap::{ArgMatches, Command};
use steamlocate::SteamDir;

use crate::{config::project::Configuration, error::Error, utils::create_link};

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
pub fn execute(matches: &ArgMatches) -> Result<(), Error> {
    let config = Configuration::from_file(&Path::new(".hemtt").join("project.toml"))?;
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
        .ok_or(Error::LaunchConfigNotFound(launch_config.to_string()))?;

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
        for load_mod in launch.workshop() {
            let mod_path = workshop_folder.join(load_mod);
            if !mod_path.exists() {
                return Err(Error::WorkshopModNotFound(load_mod.to_string()));
            };
            mods.push(mod_path.display().to_string());
        }
    }

    if !launch.dlc().is_empty() {
        for dlc in launch.dlc() {
            mods.push(dlc.to_mod().to_string());
        }
    }

    let ctx = super::dev::execute(matches, launch.optionals())?;

    let prefix_folder = arma3dir.path.join(mainprefix);
    if !prefix_folder.exists() {
        std::fs::create_dir_all(&prefix_folder)?;
    }

    let link = prefix_folder.join(ctx.config().prefix());
    if !link.exists() {
        create_link(
            link.display().to_string().as_str(),
            ctx.out_folder().display().to_string().as_str(),
        )?;
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
