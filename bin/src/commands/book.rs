use clap::{ArgMatches, Command};

use crate::{report::Report, Error};

#[must_use]
pub fn cli() -> Command {
    Command::new("book").about("Open The HEMTT book")
}

/// Execute the book command
///
/// # Errors
/// Will not return an error
pub fn execute(_: &ArgMatches) -> Result<Report, Error> {
    if let Err(e) = webbrowser::open("https://brettmayson.github.io/HEMTT/") {
        eprintln!("Failed to open the HEMTT book: {e}");
    }
    Ok(Report::new())
}
