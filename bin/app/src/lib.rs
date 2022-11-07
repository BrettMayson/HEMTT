#![deny(clippy::all, clippy::nursery)]
#![warn(clippy::pedantic)]

use clap::{ArgAction, ArgMatches, Command};
use hemtt_error::AppError;

mod addons;
mod commands;
mod context;
mod error;
mod executor;
mod modules;

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
    Command::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(commands::dev::cli())
        .subcommand(hemtt_bin_internal::cli().name("internal"))
        .arg(
            clap::Arg::new("threads")
                .global(true)
                .help("Number of threads, defaults to # of CPUs")
                .action(ArgAction::Set)
                .long("threads")
                .short('t'),
        )
}

pub fn execute(matches: &ArgMatches) -> Result<(), AppError> {
    if let Some(threads) = matches.get_one::<String>("threads") {
        if let Err(e) = rayon::ThreadPoolBuilder::new()
            .num_threads(usize::from_str_radix(&threads, 10).unwrap())
            .build_global()
        {
            println!("Failed to initialize thread pool: {}", e);
        }
    }
    match matches.subcommand() {
        Some(("dev", matches)) => commands::dev::execute(matches),
        Some(("internal", matches)) => hemtt_bin_internal::execute(matches),
        _ => unreachable!(),
    }
}
