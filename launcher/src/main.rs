use std::ffi::OsString;

use crate::github::EXECUTABLE;

pub mod github;
pub mod pr;
pub mod versions;

fn main() -> anyhow::Result<()> {
    let mut args: Vec<OsString> = std::env::args_os().skip(1).collect();

    if args.first() == Some(&OsString::from("vers")) {
        args = args[1..].to_vec();
        vers(args)?;
    } else {
        wrap(args)?;
    }
    Ok(())
}

fn wrap(mut args: Vec<OsString>) -> anyhow::Result<()> {
    // Check for a version specified by the user
    let first = args.first().cloned();
    let version = if let Some(first) = first
        && first.to_string_lossy().starts_with('+')
    {
        let version = &first.to_string_lossy()[1..];
        args = args[1..].to_vec();
        version.to_string()
    } else {
        // get the highest installed version
        let mut version = versions::installed().first().map(|v| v.to_string());
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
