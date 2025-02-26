use crate::{Error, report::Report};

#[derive(clap::Parser)]
/// Open The HEMTT book
pub struct Command {}

/// Execute the book command
///
/// # Errors
/// Will not return an error
pub fn execute(_: &Command) -> Result<Report, Error> {
    if let Err(e) = webbrowser::open("https://hemtt.dev/") {
        eprintln!("Failed to open the HEMTT book: {e}");
    }
    Ok(Report::new())
}
