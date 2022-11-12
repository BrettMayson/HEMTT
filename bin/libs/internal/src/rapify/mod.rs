use std::{
    io::Write,
    path::{Path, PathBuf},
};

use clap::{arg, value_parser, ArgMatches, Command};
use hemtt_config::{Config, Parse, Rapify};
use hemtt_error::AppError;
use hemtt_preprocessor::{preprocess_file, LocalResolver};
use peekmore::PeekMore;

pub fn cli() -> Command {
    Command::new("rapify")
        .about("Rapify on files")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("run")
                .about("Rapify a config")
                .arg_required_else_help(true)
                .arg(
                    arg!(<source> "Path to preprocess")
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    arg!(<dest> "Path to destination")
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                ),
        )
}

pub fn execute(matches: &ArgMatches) -> Result<(), AppError> {
    match matches.subcommand() {
        Some(("run", matches)) => run(
            matches.get_one::<PathBuf>("source").unwrap(),
            matches.get_one::<PathBuf>("dest").unwrap(),
        ),
        _ => unreachable!(),
    }
}

fn run(source: &Path, dest: &Path) -> Result<(), AppError> {
    assert!(source.is_file(), "Source file does not exist");
    assert!(!dest.is_file(), "Destination file already exists");
    let tokens = preprocess_file(&source.display().to_string(), &LocalResolver::new())?;
    let rapified = Config::parse(
        &hemtt_config::Options::default(),
        &mut tokens.into_iter().peekmore(),
    )?;
    let mut output = Vec::new();
    rapified.rapify(&mut output, 0).unwrap();
    let mut fs = std::fs::File::create(dest).unwrap();
    fs.write_all(&output).unwrap();
    Ok(())
}
