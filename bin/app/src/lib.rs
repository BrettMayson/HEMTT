use clap::{ArgMatches, Command};
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

pub fn cli() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(commands::dev::cli())
        .subcommand(hemtt_bin_internal::cli().name("internal"))
}

pub fn execute(matches: &ArgMatches) -> Result<(), AppError> {
    match matches.subcommand() {
        Some(("dev", matches)) => commands::dev::execute(matches),
        Some(("internal", matches)) => hemtt_bin_internal::execute(matches),
        _ => unreachable!(),
    }
}
