use std::{
    path::{Path, PathBuf},
    sync::{atomic::AtomicUsize, Arc},
};

use clap::{ArgMatches, Command};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::Error;

#[must_use]
pub fn cli() -> Command {
    Command::new("fmt").about("Format the config files").arg(
        clap::Arg::new("path")
            .help("Path to the config file or a folder to recursively fix")
            .required(true),
    )
}

/// Execute the convert command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(matches: &ArgMatches) -> Result<(), Error> {
    let path = PathBuf::from(matches.get_one::<String>("path").expect("required"));
    if path.is_dir() {
        let count = Arc::new(AtomicUsize::new(0));
        let entries = walkdir::WalkDir::new(&path)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        entries
            .par_iter()
            .map(|entry| {
                if entry.file_type().is_file()
                    && entry.path().extension().unwrap_or_default() == "cpp"
                    && entry.path().extension().unwrap_or_default() == "hpp"
                    && file(entry.path())?
                {
                    count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    info!("Format `{}`", entry.path().display());
                }
                Ok(())
            })
            .collect::<Result<Vec<_>, Error>>()?;
        info!(
            "Format {} files",
            count.load(std::sync::atomic::Ordering::Relaxed)
        );
    } else if file(&path)? {
        info!("Format `{}`", path.display());
    } else {
        info!("No changes in `{}`", path.display());
    }
    Ok(())
}

fn file(path: &Path) -> Result<bool, Error> {
    // TODO do not release this lmao
    let content = std::fs::read_to_string(path)?;
    println!(
        "{}",
        hemtt_config::fmt::format(&content).expect("errors later")
    );
    Ok(true)
}
