use std::{collections::HashMap, path::PathBuf, sync::Mutex};

use hemtt_common::arma::control::{
    fromarma::{self, Message},
    toarma,
};
use hemtt_config::{Class, Config, Property, Value};
use image::GenericImageView;

use crate::{
    context::{Context, PreservePrevious},
    controller::{Action, AutotestMission, Controller},
    error::Error,
    modules::AddonConfigs,
    report::Report,
};

mod capture;

use super::{
    JustArgs, dev,
    launch::{LaunchArgs, read_profile},
};

#[derive(clap::Parser)]
pub struct Command {
    #[arg(action = clap::ArgAction::Append, verbatim_doc_comment)]
    /// Launches with the specified profiles
    ///
    /// Configured in either:
    /// - `.hemtt/project.toml` under `hemtt.launch`
    /// - `.hemtt/launch.toml`
    config: Option<Vec<String>>,

    #[clap(flatten)]
    binarize: dev::BinarizeArgs,

    #[clap(flatten)]
    global: crate::GlobalArgs,
}

/// Execute the photoshoot command
///
/// # Errors
/// [`Error::Io`] if an IO error occurs in the Arma controller
///
/// # Panics
/// If a `dev_mission` path is set, but it has no parent
pub fn execute(cmd: &Command) -> Result<Report, Error> {
    if !dialoguer::Confirm::new()
        .with_prompt("This feature is experimental, are you sure you want to continue?")
        .interact()
        .unwrap_or_default()
    {
        return Ok(Report::new());
    }

    let mut report = Report::new();
    if cfg!(windows) && !cfg!(target_pointer_width = "64") {
        error!("Photoshoot is only supported on 64 bit Windows");
        return Ok(report);
    }

    let global = Context::read_global()?;
    let config = Context::read_project()?;
    let mut configs = cmd.config.clone().unwrap_or_default();
    if config.hemtt().launch().contains_key("photoshoot") {
        configs.push("photoshoot".to_string());
    }
    let launch = read_profile(&global, &config, &configs, &mut report);
    let Some(mut launch) = launch else {
        return Ok(report);
    };
    launch.set_mission(None);

    let (mut report, dev_ctx) = super::dev::execute(
        &dev::Command {
            global: cmd.global.clone(),
            dev: dev::DevArgs {
                optional: Vec::new(),
                all_optionals: true,
                no_rap: false,
            },
            binarize: cmd.binarize.clone(),
            just: JustArgs { just: Vec::new() },
        },
        launch.optionals(),
        launch.binarize(),
    )?;

    if let Some(dev_mission) = launch.dev_mission() {
        let dev_mission = PathBuf::from(dev_mission);
        if dev_mission.is_relative() {
            error!("dev_mission must be an absolute path");
            std::process::exit(1);
        }
        if !dev_mission.exists() {
            report.push(
                super::launch::error::bcle8_mission_not_found::MissionNotFound::code(
                    dev_mission
                        .file_name()
                        .map(|x| x.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    dev_mission.parent().expect("has parent"),
                ),
            );
        }
        warn!("dev_mission is in use: {}", dev_mission.display());
    }

    if report.failed() {
        return Ok(report);
    }
    let ctx = Context::new(Some("photoshoot"), PreservePrevious::Remove, false)?;

    let mut ps = Photoshoot::new(launch.dev_mission().map(std::string::ToString::to_string));

    ps.add_weapons(find_weapons(&dev_ctx));
    ps.add_vehicles(find_vehicles(&dev_ctx));
    ps.add_previews(find_previews(&dev_ctx));

    for (_, path) in ps
        .weapons
        .iter()
        .chain(ps.vehicles.iter().chain(ps.previews.iter()))
    {
        const ILLEGAL_CHARS: &[char] = &['<', '>', ':', '"', '/', '\n', '\t', '|', '?', '*'];
        if path.contains(ILLEGAL_CHARS) {
            // TODO add an error to the report
            error!("Path {:?} contains illegal characters", path);
            return Ok(report);
        }
    }

    if ps.weapons.is_empty() && ps.vehicles.is_empty() && ps.previews.is_empty() {
        warn!("No weapons, vehicles or previews found for photoshoot");
        return Ok(report);
    }

    let mut controller = Controller::new();
    controller.add_action(Box::new(ps));
    controller.run(&ctx, &LaunchArgs::default(), &launch)?;

    Ok(report)
}

pub struct Photoshoot {
    dev_mission: Option<String>,
    weapons: HashMap<String, String>,
    vehicles: HashMap<String, String>,
    previews: HashMap<String, String>,
    pending: Mutex<Vec<toarma::Photoshoot>>,
    capturer: Mutex<Option<capture::Capture>>,
}

impl Photoshoot {
    #[must_use]
    pub fn new(dev_mission: Option<String>) -> Self {
        Self {
            dev_mission,
            weapons: HashMap::new(),
            vehicles: HashMap::new(),
            previews: HashMap::new(),
            pending: Mutex::new(Vec::new()),
            capturer: Mutex::new(None),
        }
    }

    fn add_weapons(&mut self, weapons: HashMap<String, String>) {
        self.weapons.extend(weapons);
    }

    fn add_vehicles(&mut self, vehicles: HashMap<String, String>) {
        self.vehicles.extend(vehicles);
    }

    fn add_previews(&mut self, previews: HashMap<String, String>) {
        self.previews.extend(previews);
    }

    fn next_message(&self) -> toarma::Message {
        let mut pending = self.pending.lock().expect("pending lock");
        let message = pending.pop().map_or_else(
            || toarma::Message::Photoshoot(toarma::Photoshoot::Done),
            toarma::Message::Photoshoot,
        );
        drop(pending);
        if message == toarma::Message::Photoshoot(toarma::Photoshoot::Done) {
            std::thread::spawn(|| {
                // Allow some time for the message to be sent and processed
                std::thread::sleep(std::time::Duration::from_secs(1));
                std::process::exit(0);
            });
        }
        message
    }

    /// Initialize the screen capturer if not already initialized
    ///
    /// # Panics
    /// - If the capturer mutex is poisoned
    /// - If the capturer fails to initialize
    pub fn init_capturer(&self) {
        let mut capturer_lock = self.capturer.lock().expect("capturer lock");
        if capturer_lock.is_none() {
            *capturer_lock =
                Some(capture::Capture::new().expect("failed to initialize screen capturer"));
            drop(capturer_lock);
        }
    }

    /// Stop the screen capturer
    ///
    /// # Panics
    /// - If the capturer mutex is poisoned
    pub fn stop_capturer(&self) {
        let mut capturer_lock = self.capturer.lock().expect("capturer lock");
        *capturer_lock = None;
    }

    /// Capture a screenshot of the Arma window
    ///
    /// # Panics
    /// - If the capturer mutex is poisoned
    pub fn screenshot(&self) -> Option<image::DynamicImage> {
        self.init_capturer();
        self.capturer
            .lock()
            .expect("capturer lock")
            .as_ref()
            .expect("capturer")
            .screenshot()
    }
}

impl Action for Photoshoot {
    fn missions(&self, _: &Context) -> Vec<(String, AutotestMission)> {
        let mut missions = Vec::new();
        if !self.previews.is_empty() {
            missions.push((
                String::from("ps_previews"),
                AutotestMission::Internal(String::from("ps_previews.VR")),
            ));
        }
        if !self.weapons.is_empty() || !self.vehicles.is_empty() {
            missions.push((
                String::from("ps_items"),
                self.dev_mission.as_ref().map_or_else(
                    || AutotestMission::Internal(String::from("ps_items.VR")),
                    |dev_mission| AutotestMission::Custom(dev_mission.clone()),
                ),
            ));
        }
        missions
    }

    fn incoming(&self, _ctx: &Context, msg: fromarma::Message) -> Vec<toarma::Message> {
        self.init_capturer();
        let Message::Photoshoot(msg) = msg else {
            return Vec::new();
        };
        match &msg {
            fromarma::Photoshoot::ItemsReady => {
                debug!("Photoshoot: Items Ready");
                if self.weapons.is_empty() && self.vehicles.is_empty() {
                    return vec![self.next_message()];
                }
                let mut messages = Vec::new();
                for weapon in self.weapons.keys() {
                    messages.push(toarma::Message::Photoshoot(toarma::Photoshoot::Weapon(
                        weapon.clone(),
                    )));
                }
                for vehicle in self.vehicles.keys() {
                    messages.push(toarma::Message::Photoshoot(toarma::Photoshoot::Vehicle(
                        vehicle.clone(),
                    )));
                }
                messages
            }
            fromarma::Photoshoot::Weapon(class) | fromarma::Photoshoot::Vehicle(class) => {
                let target = if matches!(msg, fromarma::Photoshoot::Weapon(_)) {
                    debug!("Photoshoot: Weapon: {}", class);
                    PathBuf::from(
                        self.weapons
                            .get(class)
                            .expect("received unknown weapon")
                            .replace('\\', "/"),
                    )
                } else {
                    debug!("Photoshoot: Vehicle: {}", class);
                    PathBuf::from(
                        self.vehicles
                            .get(class)
                            .expect("received unknown vehicle")
                            .replace('\\', "/"),
                    )
                };
                if target.exists() {
                    warn!("Target already exists: {}", target.display());
                    return vec![self.next_message()];
                }
                // let image = utils::photoshoot::Photoshoot::weapon(class, &self.from, false)
                //     .expect("image")
                //     .into();
                let image = self.screenshot().expect("screenshot");
                let paa = hemtt_paa::Paa::from_dynamic(&image, {
                    let (width, height) = image.dimensions();
                    if !height.is_power_of_two() || !width.is_power_of_two() {
                        hemtt_paa::PaXType::ARGB8
                    } else {
                        let has_transparency = image.pixels().any(|p| p.2[3] < 255);
                        if has_transparency {
                            hemtt_paa::PaXType::DXT5
                        } else {
                            hemtt_paa::PaXType::DXT1
                        }
                    }
                })
                .expect("paa");
                fs_err::create_dir_all(target.parent().expect("has parent")).expect("create dir");
                let mut file = fs_err::File::create(target).expect("create paa");
                paa.write(&mut file).expect("write paa");
                vec![self.next_message()]
            }
            fromarma::Photoshoot::WeaponUnsupported(weapon) => {
                warn!("Photoshoot: WeaponUnsupported: {}", weapon);
                vec![self.next_message()]
            }
            fromarma::Photoshoot::VehicleUnsupported(vehicle) => {
                warn!("Photoshoot: VehicleUnsupported: {}", vehicle);
                vec![self.next_message()]
            }
            fromarma::Photoshoot::PreviewsReady => {
                debug!("Photoshoot: Previews Ready");
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
            fromarma::Photoshoot::Previews(class) => {
                debug!("Photoshoot: Preview: {}", class);
                let target = PathBuf::from(
                    self.previews
                        .get(class)
                        .expect("received unknown preview")
                        .replace('\\', "/"),
                );
                if target.exists() {
                    warn!("Target already exists: {}", target.display());
                    return vec![self.next_message()];
                }
                let image = self.screenshot().expect("screenshot");
                let image: image::DynamicImage = image::imageops::resize(
                    &image,
                    455,
                    256,
                    image::imageops::FilterType::Lanczos3,
                )
                .into();
                fs_err::create_dir_all(target.parent().expect("has parent")).expect("create dir");
                image
                    .save_with_format(target, image::ImageFormat::Jpeg)
                    .expect("save");
                vec![self.next_message()]
            }
            fromarma::Photoshoot::PreviewsDone => {
                debug!("Photoshoot: PreviewsDone");
                vec![toarma::Message::Photoshoot(toarma::Photoshoot::Done)]
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
            if !name.as_str().eq_ignore_ascii_case("cfgweapons") {
                return;
            }
            weapons.extend(find_pictures(ctx, &mainprefix, properties));
        }
    });
    weapons
}

fn find_vehicles(ctx: &Context) -> HashMap<String, String> {
    let mut vehicles = HashMap::new();
    ctx.state()
        .get::<AddonConfigs>()
        .read()
        .expect("addon configs")
        .iter()
        .for_each(|(_, configs)| {
            for (_, config) in configs {
                vehicles.extend(vehicles_from_config(ctx, config));
            }
        });
    vehicles
}

fn vehicles_from_config(ctx: &Context, config: &Config) -> HashMap<String, String> {
    let Some(mainprefix) = ctx.config().mainprefix() else {
        return HashMap::new();
    };
    let mainprefix = format!("\\{mainprefix}\\");
    let mut vehicles = HashMap::new();
    config.0.iter().for_each(|root| {
        if let Property::Class(Class::Local {
            name, properties, ..
        }) = root
        {
            if !name.as_str().eq_ignore_ascii_case("cfgvehicles") {
                return;
            }
            vehicles.extend(find_pictures(ctx, &mainprefix, properties));
        }
    });
    vehicles
}

fn find_pictures(
    ctx: &Context,
    mainprefix: &str,
    properties: &[Property],
) -> HashMap<String, String> {
    let mut pictures = HashMap::new();
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
                    if name.as_str().eq_ignore_ascii_case("picture") {
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
            if picture.starts_with(mainprefix) {
                let picture = picture.trim_start_matches(mainprefix);
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
                    pictures.insert(name.as_str().to_string(), picture.to_owned());
                }
            }
        }
    }
    pictures
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
            if !name.as_str().eq_ignore_ascii_case("cfgvehicles") {
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
                            if name.as_str().eq_ignore_ascii_case("editorpreview") {
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
