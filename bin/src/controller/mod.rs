#![allow(clippy::unwrap_used)] // Experimental feature

use std::{
    io::{Read, Write},
    process::Child,
};

use hemtt_common::{
    arma::control::{fromarma, toarma},
    config::LaunchOptions,
};
use interprocess::local_socket::{
    traits::Listener, GenericNamespaced, ListenerNonblockingMode, ListenerOptions, ToNsName,
};

use crate::{
    commands::launch::{launcher::Launcher, LaunchArgs},
    context::Context,
    error::Error,
    report::Report,
};

mod action;
mod profile;

pub use action::Action;

#[derive(Default)]
pub struct Controller {
    pub actions: Vec<Box<dyn action::Action>>,
}

impl Controller {
    #[must_use]
    pub fn new() -> Self {
        Self { actions: vec![] }
    }

    pub fn add_action(&mut self, action: Box<dyn action::Action>) {
        self.actions.push(action);
    }

    /// Run the controller
    ///
    /// # Errors
    /// - [`Error::Io`] if profile files cannot be written to disk in the temporary directory
    /// - [`Error::Io`] if there is an issue with the local socket
    ///
    /// # Panics
    /// - If an message is not able to be read from the local socket
    /// - If a message is in an unexpected format
    pub fn run(
        self,
        ctx: &Context,
        launch_args: &LaunchArgs,
        launch_options: &LaunchOptions,
    ) -> Result<Report, Error> {
        let mut missions = vec![];
        for action in &self.actions {
            action
                .missions(ctx)
                .iter()
                .for_each(|m| missions.push(m.clone()));
        }
        profile::setup(ctx)?;
        profile::autotest(ctx, &missions)?;
        let (report, child) = launch(ctx, launch_args, launch_options)?;
        let Some(mut child) = child else {
            return Ok(report);
        };
        let opts = ListenerOptions::new().name("hemtt_arma".to_ns_name::<GenericNamespaced>()?);
        let socket = opts.create_sync()?;
        socket.set_nonblocking(ListenerNonblockingMode::Both)?;
        info!("Waiting for Arma...");
        let start = std::time::Instant::now();
        let mut did_warn = false;
        let mut socket = loop {
            if let Ok(s) = socket.accept() {
                break s;
            }
            if !did_warn && start.elapsed().as_secs() > 120 {
                warn!("Still waiting after 120 seconds");
                did_warn = true;
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
                socket.read_exact(&mut buf).expect("Failed to read message");
                let buf = String::from_utf8(buf).expect("Failed to parse message");
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
                } else if let fromarma::Message::Log(level, text) = message {
                    match level {
                        fromarma::Level::Trace => trace!("arma: {}", text),
                        fromarma::Level::Debug => debug!("arma: {}", text),
                        fromarma::Level::Info => info!("arma: {}", text),
                        fromarma::Level::Warn => warn!("arma: {}", text),
                        fromarma::Level::Error => error!("arma: {}", text),
                    }
                } else if let Some(current) = &current {
                    trace!("msg for {current}: {message:?}");
                    self.actions
                        .iter()
                        .find(|a| a.missions(ctx).iter().any(|m| &m.1 == current))
                        .expect("No action for mission")
                        .incoming(ctx, message)
                        .iter()
                        .for_each(|m| send(m, &mut socket));
                } else {
                    warn!("Message without mission: {:?}", message);
                }
            }
        }
        Ok(report)
    }
}

fn launch(
    ctx: &Context,
    launch_args: &LaunchArgs,
    launch_options: &LaunchOptions,
) -> Result<(Report, Option<Child>), Error> {
    let (mut report, launcher) = Launcher::new(launch_args, launch_options)?;

    let Some(mut launcher) = launcher else {
        return Ok((report, None));
    };
    launcher.add_self_mod()?;

    let mut args: Vec<String> = ["-name=hemtt", "-window"]
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
    args.push(format!("-cfg=\"{}\\arma3.cfg\"", ctx.profile().display()));
    args.push(format!("-mod=\"{}\\@hemtt\"", ctx.profile().display()));

    let child = launcher.launch(args, &mut report)?;
    Ok((report, child))
}

#[allow(clippy::cast_possible_truncation)]
fn send(
    message: &toarma::Message,
    socket: &mut interprocess::local_socket::prelude::LocalSocketStream,
) {
    let message = serde_json::to_string(message).unwrap();
    trace!("sending: {}", message);
    socket
        .write_all(&u32::to_le_bytes(message.len() as u32))
        .unwrap();
    socket.write_all(message.as_bytes()).unwrap();
    socket.flush().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));
}
