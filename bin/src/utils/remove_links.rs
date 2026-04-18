use hemtt_common::steam;

use crate::{Error, commands::launch::error::bcle4_arma_not_found::ArmaNotFound, report::Report};

#[derive(clap::Parser)]
/// Remove file-patching links from within the Arma 3 game directory
/// 
/// ## Global Configuration
///
/// `remove_links` can be configured in the [global configuration file](/configuration/global.md).
/// 
/// When enabled, HEMTT will run `hemtt utils remove-links` before launching with `hemtt launch`. This is useful to avoid conflicts when mods have dependencies on other mods that are developed locally.
/// 
/// ```toml,fp={config}/hemtt/config.toml
/// [launch]
/// remove_links = true
/// ```
pub struct Command {}

/// Execute the `remove_links` command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If the args are not present from clap
pub fn execute(_: &Command) -> Result<Report, Error> {
    let mut report = Report::new();
    let Some(arma3) = steam::find_app(107_410) else {
        report.push(ArmaNotFound::code());
        return Ok(report);
    };

    // Traverse every file in the Arma 3 directory and remove any symbolic links
    let mut count = 0;
    for entry in walkdir::WalkDir::new(arma3).follow_links(false) {
        let entry = entry?;
        let path = entry.path();
        if path.is_symlink() {
            info!("Removing symbolic link: {}", path.display());
            fs_err::remove_file(path)?;
            count += 1;
        }
    }

    info!("Removed {} symbolic links from the Arma 3 directory", count);

    Ok(report)
}
