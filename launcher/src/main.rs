use std::ffi::OsString;
use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::github::EXECUTABLE;

pub mod github;
pub mod pr;
pub mod versions;

const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(12 * 60 * 60);
/// Marker file in the install directory, its modified time is the last successful check
const UPDATE_CHECK_FILE: &str = "last-update-check";

fn main() -> anyhow::Result<()> {
    let mut args: Vec<OsString> = std::env::args_os().skip(1).collect();

    if args.first() == Some(&OsString::from("versions"))
        || args.first() == Some(&OsString::from("v"))
    {
        args = args[1..].to_vec();
        vers(args)?;
    } else {
        wrap(args)?;
    }
    Ok(())
}

/// Install the latest release if it is newer than the highest installed version.
/// Failures are reported but never prevent HEMTT from running.
fn auto_update() {
    // Matches the update check in the HEMTT binary, which is skipped in CI
    if std::env::var_os("CI").is_some() {
        return;
    }
    if let Err(err) = try_auto_update() {
        eprintln!("Failed to update HEMTT: {err:#}");
    }
}

fn try_auto_update() -> anyhow::Result<()> {
    // Nothing installed yet, the bootstrap in `wrap` handles the first install
    let Some(installed) = versions::installed().into_iter().max() else {
        return Ok(());
    };
    let marker = versions::install_dir().join(UPDATE_CHECK_FILE);
    if !update_check_due(&marker) {
        return Ok(());
    }
    let latest = github::latest_release()?;
    fs_err::create_dir_all(versions::install_dir())?;
    fs_err::write(&marker, latest.version.to_string())?;
    if latest.version > installed {
        eprintln!("Updating HEMTT {installed} to {}...", latest.version);
        github::download_and_install(&latest)?;
    }
    Ok(())
}

fn update_check_due(marker: &Path) -> bool {
    let Ok(modified) = fs_err::metadata(marker).and_then(|m| m.modified()) else {
        return true;
    };
    SystemTime::now()
        .duration_since(modified)
        .unwrap_or_default()
        >= UPDATE_CHECK_INTERVAL
}

fn wrap(mut args: Vec<OsString>) -> anyhow::Result<()> {
    auto_update();
    // Check for a version specified by the user
    let first = args.first().cloned();
    let version = if let Some(first) = first
        && first.to_string_lossy().starts_with('+')
    {
        let version = &first.to_string_lossy()[1..];
        args = args[1..].to_vec();
        version.to_string()
    } else {
        let mut version = versions::installed()
            .into_iter()
            .max()
            .map(|v| v.to_string());
        if version.is_none() {
            // No hemtt is installed, download the latest version
            let latest = github::latest_release()?;
            github::download_and_install(&latest)?;
            version = Some(latest.version.to_string());
        }
        version.unwrap()
    };
    let hemtt_path = if let Ok(semver_version) = semver::Version::parse(&version) {
        // check if the version is installed, if not download and install it
        if !versions::installed()
            .iter()
            .any(|v| v.to_string() == version)
        {
            let release = github::get_release(&semver_version)?;
            github::download_and_install(&release)?;
        }
        versions::version_dir(&semver_version).join(EXECUTABLE)
    } else if let Some(stripped) = version.strip_prefix("pr/") {
        if !pr::can_download()? {
            anyhow::bail!("GitHub CLI (gh) is not installed or authenticated");
        }
        let pr_number: u64 = stripped.parse()?;
        pr::download_pr_build(pr_number)?;
        versions::pr_dir(pr_number).join(EXECUTABLE)
    } else {
        anyhow::bail!("Unknown version format: {}", version);
    };
    // execute the hemtt binary with the specified version
    let status = std::process::Command::new(hemtt_path)
        .args(&args)
        .status()?;
    if !status.success() {
        anyhow::bail!("HEMTT exited with status: {}", status);
    }
    Ok(())
}

fn vers(args: Vec<OsString>) -> anyhow::Result<()> {
    match args.first() {
        Some(arg) if arg == "list" => {
            for version in versions::installed() {
                println!("{}", version);
            }
        }
        Some(arg) if arg == "install" => {
            if let Some(version) = args.get(1) {
                let version =
                    github::get_release(&semver::Version::parse(&version.to_string_lossy())?)?;
                github::download_and_install(&version)?;
            } else {
                let latest = github::latest_release()?;
                github::download_and_install(&latest)?;
            }
        }
        Some(arg) if arg == "remove" => {
            if let Some(version) = args.get(1) {
                match versions::remove(&version.to_string_lossy()) {
                    Ok(true) => println!("Version removed successfully."),
                    Ok(false) => println!("Version not found."),
                    Err(err) => println!("Failed to remove version: {}", err),
                }
            } else {
                println!("Please specify a version to remove.");
            }
        }
        _ => {
            println!("Unknown vers command");
        }
    }
    Ok(())
}
