use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Mutex,
};

use hemtt_common::{
    arma::control::{
        fromarma::{self, Message},
        toarma,
    },
    config::ProjectConfig,
};
use hemtt_config::{Class, Config, Property, Value};
use image::codecs::jpeg::JpegEncoder;

use crate::{
    context::{Context, PreservePrevious},
    controller::{Action, Controller},
    error::Error,
    modules::AddonConfigs,
    report::Report,
    utils,
};

mod error;

use self::error::bcpe1_tools_not_found::ToolsNotFound;

use super::{
    dev,
    launch::{read_config, LaunchArgs},
    JustArgs,
};

#[derive(clap::Parser)]
pub struct Command {
    #[arg(action = clap::ArgAction::Append, verbatim_doc_comment)]
    /// Launches with the specified configurations
    ///
    /// Configured in either:
    /// - `.hemtt/project.toml` under `hemtt.launch`
    /// - `.hemtt/launch.toml`
    config: Option<Vec<String>>,

    #[clap(flatten)]
    global: crate::GlobalArgs,
}

#[allow(clippy::too_many_lines)]
/// Execute the photoshoot command
///
/// # Errors
/// [`Error::Io`] if an IO error occurs in the Arma controller
pub fn execute(cmd: &Command) -> Result<Report, Error> {
    if !dialoguer::Confirm::new()
        .with_prompt("This feature is experimental, are you sure you want to continue?")
        .interact()
        .unwrap_or_default()
    {
        return Ok(Report::new());
    }

    let mut report = Report::new();
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let Ok(key) = hkcu.open_subkey("Software\\Bohemia Interactive\\ImageToPAA") else {
        report.push(ToolsNotFound::code());
        return Ok(report);
    };
    let Ok(path) = key.get_value::<String, _>("tool") else {
        report.push(ToolsNotFound::code());
        return Ok(report);
    };
    let command = PathBuf::from(path);
    if !command.exists() {
        report.push(ToolsNotFound::code());
        return Ok(report);
    }

    let mut report = Report::new();
    if cfg!(windows) && !cfg!(target_pointer_width = "64") {
        error!("Photoshoot is only supported on 64 bit Windows");
        return Ok(report);
    }

    let config = ProjectConfig::from_file(&Path::new(".hemtt").join("project.toml"))?;
    let mut configs = cmd.config.clone().unwrap_or_default();
    if config.hemtt().launch().contains_key("photoshoot") {
        configs.push("photoshoot".to_string());
    }
    let launch = read_config(&config, &configs, &mut report);
    let Some(mut launch) = launch else {
        return Ok(report);
    };
    launch.set_mission(None);

    let (report, dev_ctx) = super::dev::execute(
        &dev::Command {
            global: cmd.global.clone(),
            dev: dev::DevArgs {
                optional: Vec::new(),
                all_optionals: true,
                binarize: false,
                no_rap: false,
            },
            just: JustArgs { just: Vec::new() },
        },
        launch.optionals(),
    )?;
    if report.failed() {
        return Ok(report);
    }
    let ctx = Context::new(Some("photoshoot"), PreservePrevious::Remove, false)?;

    let mut ps = Photoshoot::new(command, ctx.profile().join("Users/hemtt/Screenshots"));

    ps.add_weapons(find_weapons(&dev_ctx));
    ps.add_previews(find_previews(&dev_ctx));

    if !ps.prepare() {
        return Ok(report);
    }

    let mut controller = Controller::new();
    controller.add_action(Box::new(ps));
    controller.run(&ctx, &LaunchArgs::default(), &launch)?;

    Ok(report)
}

pub struct Photoshoot {
    weapons: HashMap<String, String>,
    previews: HashMap<String, String>,
    pending: Mutex<Vec<toarma::Photoshoot>>,
    from: PathBuf,
    command: PathBuf,
}

impl Photoshoot {
    #[must_use]
    pub fn new(command: PathBuf, from: PathBuf) -> Self {
        Self {
            command,
            from,
            weapons: HashMap::new(),
            previews: HashMap::new(),
            pending: Mutex::new(Vec::new()),
        }
    }

    fn add_weapons(&mut self, weapons: HashMap<String, String>) {
        self.weapons.extend(weapons);
    }

    fn add_previews(&mut self, previews: HashMap<String, String>) {
        self.previews.extend(previews);
    }

    fn prepare(&self) -> bool {
        let mut pending = self.pending.lock().expect("pending lock");
        pending.extend(
            self.weapons
                .keys()
                .map(|weapon| toarma::Photoshoot::Weapon(weapon.clone())),
        );
        if pending.is_empty() && self.previews.is_empty() {
            info!("No missing items to photoshoot");
            return false;
        }
        drop(pending);
        true
    }

    fn next_message(&self) -> toarma::Message {
        let mut pending = self.pending.lock().expect("pending lock");
        pending.pop().map_or_else(
            || toarma::Message::Photoshoot(toarma::Photoshoot::Done),
            toarma::Message::Photoshoot,
        )
    }
}

impl Action for Photoshoot {
    fn missions(&self, _: &Context) -> Vec<(String, String)> {
        vec![(String::from("photoshoot"), String::from("photoshoot.VR"))]
    }

    fn incoming(&self, ctx: &Context, msg: fromarma::Message) -> Vec<toarma::Message> {
        let Message::Photoshoot(msg) = msg else {
            return Vec::new();
        };
        match msg {
            fromarma::Photoshoot::Ready => {
                debug!("Photoshoot: Ready");
                if self.previews.is_empty() {
                    vec![self.next_message()]
                } else {
                    let mut messages = Vec::new();
                    for class in self.previews.keys() {
                        messages.push(toarma::Message::Photoshoot(toarma::Photoshoot::Preview(
                            class.clone(),
                        )));
                    }
                    messages.push(toarma::Message::Photoshoot(toarma::Photoshoot::PreviewRun));
                    messages
                }
            }
            fromarma::Photoshoot::Weapon(weapon) => {
                debug!("Photoshoot: Weapon: {}", weapon);
                let target =
                    PathBuf::from(self.weapons.get(&weapon).expect("received unknown weapon"));
                if target.exists() {
                    warn!("Target already exists: {}", target.display());
                    return vec![self.next_message()];
                }
                let image =
                    utils::photoshoot::Photoshoot::weapon(&weapon, &self.from).expect("image");
                let dst_png = ctx
                    .build_folder()
                    .expect("photoshoot has a folder")
                    .join(format!("{weapon}_ca.png"));
                image.save(&dst_png).expect("save");
                std::process::Command::new(&self.command)
                    .arg(dst_png)
                    .output()
                    .expect("failed to execute process");
                let dst_paa = ctx
                    .build_folder()
                    .expect("photoshoot has a folder")
                    .join(format!("{weapon}_ca.paa"));
                std::fs::create_dir_all(target.parent().expect("has parent")).expect("create dir");
                info!("Created `{}` at `{}`", weapon, target.display());
                std::fs::rename(dst_paa, target).expect("rename");
                vec![self.next_message()]
            }
            fromarma::Photoshoot::Previews => {
                debug!("Photoshoot: Previews");
                let source = self
                    .from
                    .join("EditorPreviews")
                    .join(".hemttout")
                    .join("dev");
                for image in source.read_dir().expect("read dir") {
                    let src = image.expect("image exists").path();
                    let target = PathBuf::from(
                        self.previews
                            .get(
                                &src.file_stem()
                                    .expect("has stem")
                                    .to_string_lossy()
                                    .to_string(),
                            )
                            .expect("received unknown preview"),
                    );
                    let image = utils::photoshoot::Photoshoot::preview(&src).expect("image");
                    std::fs::create_dir_all(target.parent().expect("has parent"))
                        .expect("create dir");
                    info!(
                        "Created `{}` at `{}`",
                        src.file_stem()
                            .expect("has stem")
                            .to_string_lossy()
                            .to_string(),
                        target.display()
                    );
                    let target = std::fs::File::create(target).expect("create");
                    JpegEncoder::new_with_quality(target, 90)
                        .encode(
                            &image,
                            image.width(),
                            image.height(),
                            image::ExtendedColorType::Rgb8,
                        )
                        .expect("encode");
                }
                vec![self.next_message()]
            }
        }
    }
}

fn find_weapons(ctx: &Context) -> HashMap<String, String> {
    let mut weapons = HashMap::new();
    ctx.state()
        .get::<AddonConfigs>()
        .read()
        .expect("addon configs")
        .iter()
        .for_each(|(_, configs)| {
            for (_, config) in configs {
                weapons.extend(weapons_from_config(ctx, config));
            }
        });
    weapons
}

fn weapons_from_config(ctx: &Context, config: &Config) -> HashMap<String, String> {
    let Some(mainprefix) = ctx.config().mainprefix() else {
        return HashMap::new();
    };
    let mainprefix = format!("\\{mainprefix}\\");
    let mut weapons = HashMap::new();
    config.0.iter().for_each(|root| {
        if let Property::Class(Class::Local {
            name, properties, ..
        }) = root
        {
            if name.as_str() != "CfgWeapons" {
                return;
            }
            for prop in properties {
                if let Property::Class(Class::Local {
                    name, properties, ..
                }) = prop
                {
                    trace!("Weapon: {}", name.as_str());
                    let Some(picture) = properties.iter().find_map(|prop| {
                        if let Property::Entry {
                            name,
                            value: Value::Str(value),
                            ..
                        } = prop
                        {
                            if name.as_str() == "picture" {
                                Some(value.value().to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }) else {
                        continue;
                    };
                    if picture.starts_with(&mainprefix) {
                        let picture = picture.trim_start_matches(&mainprefix);
                        if picture.starts_with(ctx.config().prefix()) {
                            let picture = picture
                                .trim_start_matches(ctx.config().prefix())
                                .trim_start_matches('\\');
                            let image = ctx
                                .workspace_path()
                                .join(picture.replace('\\', "/"))
                                .expect("workspace path");
                            if image.exists().unwrap_or_default() {
                                continue;
                            }
                            debug!("Image not found: {}", image.as_str());
                            weapons.insert(name.as_str().to_string(), picture.to_owned());
                        }
                    }
                }
            }
        }
    });
    weapons
}

fn find_previews(ctx: &Context) -> HashMap<String, String> {
    let mut previews = HashMap::new();
    ctx.state()
        .get::<AddonConfigs>()
        .read()
        .expect("addon configs")
        .iter()
        .for_each(|(_, configs)| {
            for (_, config) in configs {
                previews.extend(previews_from_config(ctx, config));
            }
        });
    previews
}

fn previews_from_config(ctx: &Context, config: &Config) -> HashMap<String, String> {
    let Some(mainprefix) = ctx.config().mainprefix() else {
        return HashMap::new();
    };
    let mainprefix = format!("\\{mainprefix}\\");
    let mut weapons = HashMap::new();
    config.0.iter().for_each(|root| {
        if let Property::Class(Class::Local {
            name, properties, ..
        }) = root
        {
            if name.as_str() != "CfgVehicles" {
                return;
            }
            for prop in properties {
                if let Property::Class(Class::Local {
                    name, properties, ..
                }) = prop
                {
                    trace!("Preview: {}", name.as_str());
                    let Some(picture) = properties.iter().find_map(|prop| {
                        if let Property::Entry {
                            name,
                            value: Value::Str(value),
                            ..
                        } = prop
                        {
                            if name.as_str() == "editorPreview" {
                                Some(value.value().to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }) else {
                        continue;
                    };
                    if picture.starts_with(&mainprefix) {
                        let picture = picture.trim_start_matches(&mainprefix);
                        if picture.starts_with(ctx.config().prefix()) {
                            let picture = picture
                                .trim_start_matches(ctx.config().prefix())
                                .trim_start_matches('\\');
                            let image = ctx
                                .workspace_path()
                                .join(picture.replace('\\', "/"))
                                .expect("workspace path");
                            if image.exists().unwrap_or_default() {
                                continue;
                            }
                            debug!("Image not found: {}", image.as_str());
                            weapons.insert(name.as_str().to_string(), picture.to_owned());
                        }
                    }
                }
            }
        }
    });
    weapons
}
