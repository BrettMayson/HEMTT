#![deny(clippy::all, clippy::nursery)]
#![warn(clippy::pedantic)]

use clap::{ArgAction, ArgMatches, Command};
use context::Context;
use hemtt_error::AppError;

#[macro_use]
extern crate tracing;

mod addons;
mod commands;
mod config;
mod context;
mod error;
mod executor;
mod logging;
mod modules;
mod update;
mod utils;

#[must_use]
pub fn cli() -> Command {
    #[allow(unused_mut)]
    let mut global = Command::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(false)
        .arg_required_else_help(true)
        .subcommand(commands::new::cli())
        .subcommand(commands::dev::cli())
        .subcommand(commands::build::cli())
        .subcommand(commands::release::cli())
        .subcommand(commands::launch::cli());
    global = global.arg(
        clap::Arg::new("threads")
            .global(true)
            .help("Number of threads, defaults to # of CPUs")
            .action(ArgAction::Set)
            .long("threads")
            .short('t'),
    );
    global = global.arg(
        clap::Arg::new("verbosity")
            .global(true)
            .help("Verbosity level")
            .action(ArgAction::Count)
            .short('v'),
    );
    global = global.arg(
        clap::Arg::new("trace")
            .global(true)
            .help("Enable trace logging")
            .action(ArgAction::SetTrue)
            .long("trace"),
    );
    #[cfg(debug_assertions)]
    {
        global = global.arg(
            clap::Arg::new("in-test")
                .global(true)
                .help("we are in a test")
                .action(ArgAction::SetTrue)
                .long("in-test"),
        );
    }
    global
}

/// Run the HEMTT CLI
///
/// # Errors
/// If the command fails
///
/// # Panics
/// If the number passed to `--threads` is not a valid number
pub fn execute(matches: &ArgMatches) -> Result<(), AppError> {
    if cfg!(not(debug_assertions)) || !matches.get_flag("in-test") {
        logging::init(matches.get_count("verbosity"), matches.get_flag("trace"));
    }

    match update::check() {
        Ok(Some(version)) => {
            info!("HEMTT {version} is available, please update");
        }
        Err(e) => {
            error!("Failed to check for updates: {e}");
        }
        _ => {}
    }

    trace!("version: {}", env!("CARGO_PKG_VERSION"));
    trace!("platform: {}", std::env::consts::OS);

    if let Some(threads) = matches.get_one::<String>("threads") {
        if let Err(e) = rayon::ThreadPoolBuilder::new()
            .num_threads(threads.parse::<usize>().unwrap())
            .build_global()
        {
            error!("Failed to initialize thread pool: {e}");
        }
    }
    match matches.subcommand() {
        Some(("new", matches)) => commands::new::execute(matches).map_err(std::convert::Into::into),
        Some(("dev", matches)) => {
            commands::dev::execute(matches)?;
            Ok(())
        }
        Some(("build", matches)) => {
            let ctx = Context::new("build")?;
            let mut executor = executor::Executor::new(&ctx);
            commands::build::execute(matches, &mut executor).map_err(std::convert::Into::into)
        }
        Some(("release", matches)) => {
            commands::release::execute(matches).map_err(std::convert::Into::into)
        }
        Some(("launch", matches)) => {
            commands::launch::execute(matches).map_err(std::convert::Into::into)
        }
        _ => unreachable!(),
    }
}
