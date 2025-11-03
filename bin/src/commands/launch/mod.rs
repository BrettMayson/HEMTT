use hemtt_common::config::{GlobalConfig, LaunchOptions, ProjectConfig};
use launcher::Launcher;

use crate::{
    commands::launch::error::{
        bcle5_missing_main_prefix::MissingMainPrefix,
        bcle6_launch_profile_not_found::{LaunchProfileNotFound, LaunchSource},
        bcle7_can_not_quicklaunch::CanNotQuickLaunch,
    },
    context::Context,
    error::Error,
    report::Report,
};

pub mod error;

pub mod launcher;
mod platforms;
pub mod preset;

#[derive(clap::Parser)]
#[command(verbatim_doc_comment)]
/// Test your project
///
/// `hemtt launch` is used to build and launch a dev version of your mod.
/// It will run the [`hemtt dev`](dev.md) command internally after a
/// few checks, options are passed to the `dev` command.
///
/// ## Workflow
///
/// 1. Builds your mod using `hemtt dev`
/// 2. Loads configured Workshop mods and DLCs
/// 3. Applies HTML presets if specified
/// 4. Launches Arma 3 with all configured parameters
///
/// ## Configuration
///
/// `hemtt launch` requires the [`mainprefix`](../configuration/index.md#main-prefix) option to be set.
///
/// Launch profiles can be stored in either `.hemtt/project.toml` under `hemtt.launch`,
/// or in a separate file under `.hemtt/launch.toml`. The latter is useful for keeping
/// your main configuration file clean. When using `launch.toml`,
/// the `hemtt.launch` key is not required.
///
/// **.hemtt/project.toml**
///
/// ```toml
/// mainprefix = "z"
///
/// # Launched with `hemtt launch`
/// [hemtt.launch.default]
/// workshop = [
///     "450814997", # CBA_A3's Workshop ID
/// ]
/// presets = [
///     "main", # .html presets from .hemtt/presets/
/// ]
/// dlc = [
///     "Western Sahara",
/// ]
/// optionals = [
///     "caramel",
/// ]
/// mission = "test.VR" # Mission to launch directly into the editor with
/// parameters = [
///     "-skipIntro",           # These parameters are passed to the Arma 3 executable
///     "-noSplash",            # They do not need to be added to your list
///     "-showScriptErrors",    # You can add additional parameters here
///     "-debug",
///     "-filePatching",
/// ]
/// executable = "arma3" # Default: "arma3_x64"
/// file_patching = false # Default: true
/// binarize = true # Default: false
/// rapify = false # Default: true
///
/// # Launched with `hemtt launch vn`
/// [hemtt.launch.vn]
/// extends = "default"
/// dlc = [
///     "S.O.G. Prairie Fire",
/// ]
///
/// # Launched with `hemtt launch ace`
/// [hemtt.launch.ace]
/// extends = "default"
/// workshop = [
///     "463939057", # ACE3's Workshop ID
/// ]
/// ```
///
/// **.hemtt/launch.toml**
///
/// ```toml
/// [default]
/// workshop = [
///     "450814997", # CBA_A3's Workshop ID
/// ]
///
/// [vn]
/// extends = "default"
/// dlc = [
///     "S.O.G. Prairie Fire",
/// ]
/// ```
///
/// ### extends
///
/// The name of another profile to extend. This will merge all
/// arrays with the base profile, and override any duplicate keys.
///
/// ### workshop
///
/// A list of workshop IDs to launch with your mod. These are not
/// subscribed to, and will need to be manually subscribed to in Steam.
///
/// ### presets
///
/// A list of `.html` presets to launch with your mod.
/// Exported from the Arma 3 Launcher, and kept in `.hemtt/presets/`.
///
/// ### dlc
///
/// A list of DLCs to launch with your mod. The fullname or short-code can be used.
///
/// Currently supported DLCs:
///
/// | Full Name           | Short Code |
/// | ------------------- | ---------- |
/// | Contact             | contact    |
/// | Global Mobilization | gm         |
/// | S.O.G. Prairie Fire | vn         |
/// | CSLA Iron Curtain   | csla       |
/// | Western Sahara      | ws         |
/// | Spearhead 1944      | spe        |
/// | Reaction Forces     | rf         |
///
/// ### optionals
///
/// A list of optional addon folders to launch with your mod.
///
/// ### mission
///
/// The mission to launch directly into the editor with. This can be specified
/// as either the name of a folder in `.hemtt/missions/`
/// (e.g., `test.VR` would launch `.hemtt/missions/test.VR/mission.sqm`)
/// or the relative (to the project root) path to a `mission.sqm`
/// file or a folder containing it.
///
/// ### parameters
///
/// A list of [Startup Parameters](https://community.bistudio.com/wiki/Arma_3:_Startup_Parameters) to pass to the Arma 3 executable.
///
/// ### executable
///
/// The name of the Arma 3 executable to launch.
/// This is usually `arma3` or `arma3_x64`.
/// Do not include the `.exe` extension, it will be added automatically on Windows.
/// Only paths relative to the Arma 3 directory are supported.
///
/// ### `file_patching`
///
/// Whether to launch Arma 3 with `-filePatching`. Equivalent to `--no-filepatching` or `-F`.
///
/// ### binarize
///
/// Whether to use BI's binarize on supported files. Equivalent to `--binarize`.
///
/// ### rapify
///
/// Provides the ability to disable rapify for the launch command. Equivalent to `--no-rap`.
///
/// ## Profile Chaining
///
/// You can chain multiple profiles together, and they will be
/// overlayed from left to right. Any arrays will be concatenated,
/// and any duplicate keys will be overridden. With the above configuration,
/// `hemtt launch default vn ace` would launch with all three profiles.
/// Note that `default` must be specified when specifying additional
/// profiles, `default` is only implied when no profiles are specified.
///
/// ## CDLC Launch
///
/// To launch with a CDLC, you can avoid creating a launch profile for each CDLC and instead use `+<cdlc name>`
///
/// ```bash
/// hemtt launch my_profile +ws
/// ```
///
/// ## Global Configuration
///
/// Launch configuration can be stored in the [global configuration file](/configuration/global.md).
///
/// ### Global Profiles
///
/// Global profiles can be created to easily be used on any project on your system. The supported options are:
///
/// - `workshop`
/// - `dlc`
/// - `parameters`
/// - `executable`
///
/// These profiles can be used by prefixing the name with `@`, for example:
///
/// **{config}/hemtt/config.toml**
/// ```toml
/// [launch.profiles.adt]
/// workshop = [ "3499977893" ]
/// ```
///
/// ```bash
/// hemtt launch @adt
/// ```
///
/// ### Pointers
///
/// Global pointers can be used to
/// 1. Define a location for a workshop mod
/// 2. Define a location for non-workshop mods
///
/// **{config}/hemtt/config.toml**
/// ```toml
/// [launch.pointers]
/// my_unit = "D:\\Swifty\\MyUnit"
/// 463939057 = "D:\\Projects\\ACE3"
/// ```
///
/// Workshop pointers will automatically be used globally when the workshop ID is specified in a launch profile.
///
/// Non-workshop pointers can be used by prefixing the id of the mod in the `workshop` list with the name of the pointer.
///
/// Pointers must be at least 2 characters long to avoid conflicts with drive letters.
///
/// **.hemtt/project.toml**
/// ```toml
/// [hemtt.launch.ace]
/// workshop = [
///     "450814997", # CBA_A3's Workshop ID, will load from the workshop folder
///     "463939057", # ACE3's Workshop ID, will load from the defined pointer
/// ]
///
/// [hemtt.launch.myunit]
/// workshop = [
///     "450814997", # CBA_A3
///     "my_unit:@my_unit_gear",  # These two mods will load from the defined pointer
///     "my_unit:@my_unit_units",
/// ]
/// ```
pub struct Command {
    #[clap(flatten)]
    launch: LaunchArgs,

    #[clap(flatten)]
    dev: super::dev::DevArgs,

    #[clap(flatten)]
    binarize: super::dev::BinarizeArgs,

    #[clap(flatten)]
    just: super::JustArgs,

    #[clap(flatten)]
    global: crate::GlobalArgs,
}

#[derive(Default, clap::Args)]
#[allow(clippy::module_name_repetitions)]
pub struct LaunchArgs {
    #[arg(action = clap::ArgAction::Append, verbatim_doc_comment)]
    /// Launches with the specified configurations
    ///
    /// Configured in either:
    /// - `.hemtt/project.toml` under `hemtt.launch`
    /// - `.hemtt/launch.toml`
    config: Option<Vec<String>>,
    #[arg(long, short, verbatim_doc_comment)]
    /// Executable to launch, defaults to `arma3_x64.exe`
    ///
    /// Overrides the `executable` option in the configuration file.
    ///
    /// Can be either a relative path to the Arma 3 directory, or an absolute path.
    ///
    /// ```bash
    /// -e arma3profiling_x64 # Relative to the Arma 3 directory
    /// -e "C:\Program Files\Steam\steamapps\common\Arma 3\arma3_x64.exe" # Absolute path
    /// ```
    executable: Option<String>,
    #[arg(raw = true, verbatim_doc_comment)]
    /// Passthrough additional arguments to Arma 3
    ///
    /// Any options after `--` will be passed to the Arma 3 executable.
    /// This is useful for passing additional [Startup Parameters](https://community.bistudio.com/wiki/Arma_3:_Startup_Parameters).
    ///
    /// ```bash
    /// hemtt launch -- -world=empty -window
    /// ```
    passthrough: Option<Vec<String>>,
    #[arg(long, short)]
    /// Launches multiple instances of the game
    ///
    /// If unspecified, it will default to 1.
    instances: Option<u8>,
    #[arg(long = "quick", short = 'Q')]
    /// Skips the build step, launching the last built version
    ///
    /// Will throw an error if no build has been made, or no symlink exists.
    no_build: bool,
    #[arg(long = "no-filepatching", short = 'F')]
    /// Disables file patching
    no_filepatching: bool,

    #[arg(long = "dry-run", hide = true)]
    /// Performs a dry run of the launch command
    dry_run: bool,
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
    let mut report = Report::new();
    let global = Context::read_global()?;
    let config = Context::read_project()?;
    let Some(mainprefix) = config.mainprefix() else {
        report.push(MissingMainPrefix::code());
        return Ok(report);
    };

    let launch = read_profile(
        &global,
        &config,
        cmd.launch.config.as_deref().unwrap_or_default(),
        &mut report,
    );
    let Some(launch) = launch else {
        return Ok(report);
    };

    trace!("launch config: {:?}", launch);

    let (mut report, launcher) = Launcher::new(&global, &cmd.launch, &launch)?;

    let Some(mut launcher) = launcher else {
        return Ok(report);
    };

    launcher.add_self_mod()?;

    if cmd.launch.no_build {
        warn!("Using Quick Launch! HEMTT will not rebuild the project");
        if !std::env::current_dir()?.join(".hemttout/dev").exists() {
            report.push(CanNotQuickLaunch::code(
                "no dev build found in .hemttout/dev".to_string(),
            ));
            return Ok(report);
        }

        let prefix_folder = launcher.arma3dir().join(mainprefix);
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
            &cmd.binarize,
            &cmd.just,
            launch.optionals(),
            launch.binarize(),
            launch.rapify(),
        )?;

        report.merge(executor.run()?);

        if report.failed() {
            return Ok(report);
        }
    }

    launcher.launch(Vec::new(), cmd.launch.dry_run, &mut report)?;

    Ok(report)
}

/// Read a launch profile
pub fn read_profile(
    global: &GlobalConfig,
    config: &ProjectConfig,
    profiles: &[String],
    report: &mut Report,
) -> Option<LaunchOptions> {
    let launch = if profiles.is_empty() || profiles.iter().all(|c| c.starts_with('+')) {
        config
            .hemtt()
            .launch()
            .get("default")
            .cloned()
            .unwrap_or_default()
    } else if let Some(launch) = profiles
        .iter()
        .map(|c| {
            if let Some(gc) = c.strip_prefix("@") {
                global.launch().profiles().get(gc).cloned().map_or_else(
                    || {
                        report.push(LaunchProfileNotFound::code(
                            LaunchSource::Global,
                            gc.to_string(),
                            &global
                                .launch()
                                .profiles()
                                .keys()
                                .cloned()
                                .collect::<Vec<_>>(),
                        ));
                        None
                    },
                    Some,
                )
            } else if let Some(cdlc) = c.strip_prefix("+") {
                LaunchOptions::new_cdlc(cdlc).map_or_else(
                    |_| {
                        report.push(LaunchProfileNotFound::code(
                            LaunchSource::CDLC,
                            cdlc.to_string(),
                            &[],
                        ));
                        None
                    },
                    Some,
                )
            } else {
                config.hemtt().launch().get(c).cloned().map_or_else(
                    || {
                        report.push(LaunchProfileNotFound::code(
                            LaunchSource::Project,
                            c.clone(),
                            &config.hemtt().launch().keys().cloned().collect::<Vec<_>>(),
                        ));
                        None
                    },
                    Some,
                )
            }
        })
        .collect::<Option<Vec<_>>>()
    {
        launch.into_iter().fold(
            LaunchOptions::default(),
            hemtt_common::config::LaunchOptions::overlay,
        )
    } else {
        return None;
    };
    Some(launch)
}
