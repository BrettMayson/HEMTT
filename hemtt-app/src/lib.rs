use std::collections::HashMap;
use std::time::Instant;

#[macro_use]
extern crate log;

use clap::App;
use hemtt::{HEMTTError, Project};

mod ci;
mod command;
mod commands;
mod startup;
mod tasks;

use command::Command;

lazy_static::lazy_static! {
    pub static ref CI: bool = std::env::args().any(|x| x == "--ci") || ci::is_ci();
    pub static ref DEBUG: bool = std::env::args().any(|x| x == "--debug");
    pub static ref TRACE: bool = std::env::args().any(|x| x == "--trace");

    pub static ref VERSION: &'static str = {
        let mut version = env!("CARGO_PKG_VERSION").to_string();
        if let Some(v) = option_env!("GIT_HASH") {
            version.push_str("-");
            version.push_str(v);
        }
        if cfg!(debug_assertions) {
            version.push_str("-debug");
        }
        Box::leak(Box::new(version))
    };
}

static GIT_IGNORE: [&str; 4] = ["releases/*", "*.biprivatekey", "keys/*", ".hemtt/local*"];

pub fn execute(input: &[String], root: bool) -> Result<(), HEMTTError> {
    let mut app = App::new("HEMTT")
        .version(*crate::VERSION)
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            clap::Arg::with_name("debug")
                .global(true)
                .help("Turn debugging information on")
                .long("debug"),
        )
        .arg(
            clap::Arg::with_name("trace")
                .global(true)
                .help("Turn trace information on")
                .long("trace"),
        )
        .arg(
            clap::Arg::with_name("time")
                .global(true)
                .help("Time the execution")
                .long("time"),
        );

    let mut commands: Vec<Box<dyn Command>> = Vec::new();
    let mut hash_commands: HashMap<String, &Box<dyn Command>> = HashMap::new();

    commands.push(Box::new(commands::Bug {}));
    commands.push(Box::new(commands::Project {}));
    commands.push(Box::new(commands::Template {}));

    for command in commands.iter() {
        let sub = command.register();
        hash_commands.insert(sub.get_name().to_owned(), command);
        app = app.subcommand(sub);
    }

    let matches = app.get_matches_from(input);

    let start = if matches.is_present("time") {
        Some(Instant::now())
    } else {
        None
    };

    match matches.subcommand_name() {
        Some(v) => match hash_commands.get(v) {
            Some(c) => {
                let sub_matches = matches.subcommand_matches(v).unwrap();
                if root && c.can_announce() {
                    info!("HEMTT {}", *crate::VERSION);
                }
                if c.require_project() {
                    let project = Project::read()?;
                    // info!("Environment: {}", project::environment());
                    if root && c.can_announce() {
                        info!("{} {}", project.name, project.version);
                        startup::startup();
                    }
                    c.run(sub_matches, project)?;
                } else {
                    c.run_no_project(sub_matches)?;
                }
            }
            None => error!("No command"),
        },
        None => error!("No command"),
    }

    if matches.is_present("time") {
        let elapsed = start.unwrap().elapsed();
        info!(
            "Execution Took {}.{} Seconds",
            elapsed.as_secs(),
            elapsed.as_millis()
        );
    }

    Ok(())
}

pub fn log_path(new: bool) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    if new {
        path.push("hemtt.log");
        if path.exists() {
            if let Err(e) = std::fs::rename(&path, log_path(false)) {
                error!("error moving old path: {}", e);
            };
        }
    } else {
        path.push("hemtt.previous.log")
    }
    path
}
