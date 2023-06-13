use std::path::{Path, PathBuf};

use clap::{ArgAction, ArgMatches, Command};
use hemtt_arma::messages::{
    fromarma::{self, Message},
    toarma,
};

use crate::{
    config::project::Configuration,
    context::Context,
    controller::{Action, Controller},
    error::Error,
    utils,
};

use super::dev;

#[must_use]
pub fn cli() -> Command {
    dev::add_args(
        Command::new("photoshoot").about("Take picture").arg(
            clap::Arg::new("uniform")
                .help("A uniform to take a picture of")
                .action(ArgAction::Append),
        ),
    )
}

#[allow(clippy::too_many_lines)]
pub fn execute(matches: &ArgMatches) -> Result<(), Error> {
    if cfg!(windows) && !cfg!(target_pointer_width = "64") {
        error!("Photoshoot is only supported on 64 bit Windows");
        return Ok(());
    }

    let config = Configuration::from_file(&Path::new(".hemtt").join("project.toml"))?;
    let options = config
        .hemtt()
        .launch("photoshoot")
        .or_else(|| config.hemtt().launch("default"))
        .ok_or(Error::LaunchConfigNotFound(String::from(
            "photoshoot / default",
        )))?;

    let Some(uniforms) = matches.get_many::<String>("uniform") else {
        return Err(Error::Arma3NotFound);
    };
    let uniforms = uniforms.cloned().collect::<Vec<_>>();

    super::dev::execute(matches, options.optionals())?;
    let ctx = Context::new("photoshoot")?;
    let mut controller = Controller::new();
    controller.add_action(Box::new(Photoshoot::new(
        uniforms,
        ctx.profile().join("Users/hemtt/Screenshots"),
        ctx.out_folder().clone(),
    )));
    controller.run(&ctx, &options)?;

    Ok(())
}

pub struct Photoshoot {
    uniforms: Vec<String>,
    from: PathBuf,
    to: PathBuf,
}

impl Photoshoot {
    pub fn new(uniforms: Vec<String>, from: PathBuf, to: PathBuf) -> Self {
        Self { uniforms, from, to }
    }
}

impl Action for Photoshoot {
    fn missions(&self) -> Vec<(String, String)> {
        vec![(String::from("photoshoot"), String::from("photoshoot.VR"))]
    }

    fn incoming(&self, msg: fromarma::Message) -> Vec<toarma::Message> {
        let Message::Photoshoot(msg) = msg else {
            return Vec::new();
        };
        match msg {
            fromarma::Photoshoot::Ready => {
                let mut messages = Vec::new();
                for uniform in &self.uniforms {
                    messages.push(toarma::Message::Photoshoot(toarma::Photoshoot::Uniform(
                        uniform.clone(),
                    )));
                }
                messages.push(toarma::Message::Photoshoot(toarma::Photoshoot::Done));
                messages
            }
            fromarma::Photoshoot::Uniform(uniform) => {
                utils::Photoshoot::image(&uniform, &self.from, &self.to).unwrap();
                Vec::new()
            }
        }
    }
}
