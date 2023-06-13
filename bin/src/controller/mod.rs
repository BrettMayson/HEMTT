use std::{
    io::{Read, Write},
    process::Child,
};

use hemtt_arma::messages::{fromarma, toarma};
use steamlocate::SteamDir;

use crate::{config::project::LaunchOptions, context::Context, error::Error};

mod action;
mod profile;

pub use action::Action;

pub struct Controller {
    pub actions: Vec<Box<dyn action::Action>>,
}

impl Controller {
    pub fn new() -> Self {
        Self { actions: vec![] }
    }

    pub fn add_action(&mut self, action: Box<dyn action::Action>) {
        self.actions.push(action);
    }

    pub fn run(self, ctx: &Context, options: &LaunchOptions) -> Result<(), Error> {
        let mut missions = vec![];
        for action in &self.actions {
            action
                .missions()
                .iter()
                .for_each(|m| missions.push(m.clone()));
        }
        profile::setup(ctx)?;
        profile::autotest(ctx, &missions)?;
        let mut child = launch(ctx, options)?;
        let socket = interprocess::local_socket::LocalSocketListener::bind("hemtt_arma")?;
        info!("Waiting for Arma...");
        socket.set_nonblocking(true)?;
        let start = std::time::Instant::now();
        let mut socket = loop {
            if let Ok(s) = socket.accept() {
                break s;
            }
            if start.elapsed().as_secs() > 30 {
                return Err(Error::ControllerTimeout);
            }
        };

        info!("Connected!");

        let mut current = None;

        loop {
            let status = child.try_wait();
            if status.is_err() {
                warn!("No longer able to determine Arma's status");
                break;
            }
            if let Ok(Some(_)) = status {
                info!("Arma has exited");
                break;
            }

            let mut len_buf = [0u8; 4];
            if socket.read_exact(&mut len_buf).is_ok() && !len_buf.is_empty() {
                let len = u32::from_le_bytes(len_buf);
                trace!("Receiving: {}", len);
                let mut buf = vec![0u8; len as usize];
                socket.read_exact(&mut buf).unwrap();
                let buf = String::from_utf8(buf).unwrap();
                let message: fromarma::Message = serde_json::from_str(&buf)?;
                trace!("Received: {:?}", message);
                if let fromarma::Message::Control(control) = message {
                    match control {
                        fromarma::Control::Mission(mission) => {
                            if let Some((_, mission)) = mission.split_once("\\autotest\\") {
                                debug!("Mission: {}", mission);
                                current = Some(mission.replace('\\', ""));
                            } else {
                                warn!("Invalid mission: {}", mission);
                            }
                        }
                    }
                } else if let Some(current) = &current {
                    trace!("msg for {current}: {message:?}");
                    self.actions
                        .iter()
                        .find(|a| a.missions().iter().any(|m| &m.1 == current))
                        .unwrap()
                        .incoming(message)
                        .iter()
                        .for_each(|m| send(m, &mut socket));
                } else {
                    warn!("Message without mission: {:?}", message);
                }
            }
        }
        Ok(())
    }
}

fn launch(ctx: &Context, options: &LaunchOptions) -> Result<Child, Error> {
    let Some(arma3dir) = SteamDir::locate().and_then(|mut s| s.app(&107_410).map(std::borrow::ToOwned::to_owned)) else {
        return Err(Error::Arma3NotFound);
    };
    let mut mods = Vec::new();
    mods.push({
        let mut path = std::env::current_dir()?;
        path.push(".hemttout/dev");
        path.display().to_string()
    });

    // climb to the workshop folder
    if !options.workshop().is_empty() {
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
        for load_mod in options.workshop() {
            let mod_path = workshop_folder.join(load_mod);
            if !mod_path.exists() {
                return Err(Error::WorkshopModNotFound(load_mod.to_string()));
            };
            mods.push(mod_path.display().to_string());
        }
    }

    if !options.dlc().is_empty() {
        for dlc in options.dlc() {
            mods.push(dlc.to_mod().to_string());
        }
    }

    let mut args: Vec<String> = vec![
        "-debugCallExtension",
        "-skipIntro",
        "-noSplash",
        "-name=hemtt",
        "-window",
    ]
    .iter()
    .map(std::string::ToString::to_string)
    .collect();
    args.push(format!(
        "-autotest={}",
        ctx.profile()
            .join("Users/hemtt/autotest.cfg")
            .display()
            .to_string()
            .replace('/', "\\")
    ));
    args.insert(0, format!("-profiles={}", ctx.profile().display()));
    args.push(format!("-cfg={}\\arma3.cfg", ctx.profile().display()));
    args.push(format!("-mod={}\\@hemtt", ctx.profile().display()));
    args.push(format!("-mod=\"{}\"", mods.join(";"),));
    args.append(&mut options.parameters().to_vec());

    info!(
        "Launching {:?} with: {:?}",
        arma3dir.path.display(),
        args.join(" ")
    );

    std::process::Command::new({
        let mut path = arma3dir.path.join("arma3_x64");
        if cfg!(windows) {
            path.set_extension("exe");
        }
        path.display().to_string()
    })
    .args(args)
    .spawn()
    .map_err(std::convert::Into::into)
}

#[allow(clippy::cast_possible_truncation)]
fn send(message: &toarma::Message, socket: &mut interprocess::local_socket::LocalSocketStream) {
    let message = serde_json::to_string(message).unwrap();
    socket
        .write_all(&u32::to_le_bytes(message.len() as u32))
        .unwrap();
    socket.write_all(message.as_bytes()).unwrap();
}
