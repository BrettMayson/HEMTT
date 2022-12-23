#![deny(clippy::all, clippy::nursery)]
#![warn(clippy::pedantic)]

use clap::{ArgAction, ArgMatches, Command};
use hemtt_error::AppError;

mod addons;
mod commands;
mod context;
mod executor;
mod modules;
mod utils;

lazy_static::lazy_static! {
    pub static ref VERSION: &'static str = {
        let mut version = env!("CARGO_PKG_VERSION").to_string();
        if let Some(v) = option_env!("GIT_HASH") {
            version.push('-');
            version.push_str(v);
        }
        if cfg!(debug_assertions) {
            version.push_str("-debug");
        }
        Box::leak(Box::new(version))
    };
}

#[must_use]
pub fn cli() -> Command {
    #[allow(unused_mut)]
    let mut global = Command::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(commands::new::cli())
        .subcommand(commands::dev::cli())
        .subcommand(commands::build::cli())
        .subcommand(commands::release::cli())
        .subcommand(hemtt_bin_internal::cli().name("internal"));
    #[cfg(windows)]
    {
        global = global.subcommand(commands::launch::cli());
    }
    global.arg(
        clap::Arg::new("threads")
            .global(true)
            .help("Number of threads, defaults to # of CPUs")
            .action(ArgAction::Set)
            .long("threads")
            .short('t'),
    )
}

/// Run the HEMTT CLI
///
/// # Errors
/// If the command fails
///
/// # Panics
/// If the number passed to `--threads` is not a valid number
pub fn execute(matches: &ArgMatches) -> Result<(), AppError> {
    if let Some(threads) = matches.get_one::<String>("threads") {
        if let Err(e) = rayon::ThreadPoolBuilder::new()
            .num_threads(threads.parse::<usize>().unwrap())
            .build_global()
        {
            println!("Failed to initialize thread pool: {e}");
        }
    }
    match matches.subcommand() {
        Some(("new", matches)) => commands::new::execute(matches).map_err(std::convert::Into::into),
        Some(("dev", matches)) => {
            commands::dev::execute(matches)?;
            Ok(())
        }
        Some(("build", matches)) => {
            commands::build::execute(matches).map_err(std::convert::Into::into)
        }
        Some(("release", matches)) => {
            commands::release::execute(matches).map_err(std::convert::Into::into)
        }
        Some(("internal", matches)) => hemtt_bin_internal::execute(matches),
        #[cfg(windows)]
        Some(("launch", matches)) => {
            commands::launch::execute(matches).map_err(std::convert::Into::into)
        }
        _ => unreachable!(),
    }
}
