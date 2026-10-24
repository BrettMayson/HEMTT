use std::time::Duration;

use anyhow::{Context, Result, bail};
use fs_err::File;
use reqwest::blocking::Client;
use semver::Version;
use serde::Deserialize;
use zip::ZipArchive;

use crate::versions::{install_dir, version_dir};

const REPO: &str = "BrettMayson/hemtt";
const API: &str = "https://api.github.com";

#[cfg(target_os = "windows")]
const ASSET: &str = "windows-x64.zip";

#[cfg(target_os = "linux")]
const ASSET: &str = "linux-x64.zip";

#[cfg(target_os = "macos")]
const ASSET: &str = "darwin-arm64.zip";

#[cfg(target_os = "windows")]
pub const EXECUTABLE: &str = "hemtt.exe";

#[cfg(target_os = "linux")]
pub const EXECUTABLE: &str = "hemtt";

#[cfg(target_os = "macos")]
pub const EXECUTABLE: &str = "hemtt";

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Clone)]
pub struct ReleaseVersion {
    pub version: Version,
    pub tag: String,
    pub url: String,
}

fn client() -> Result<Client> {
    Ok(Client::builder()
        .user_agent(concat!("hemtt-launcher/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(120))
        .build()?)
}

fn parse_release(release: Release) -> Option<ReleaseVersion> {
    if release.draft || release.prerelease {
        return None;
    }
    let version_text = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name);
    let version = Version::parse(version_text).ok()?;
    let asset = release.assets.into_iter().find(|a| a.name == ASSET)?;
    Some(ReleaseVersion {
        version,
        tag: release.tag_name,
        url: asset.browser_download_url,
    })
}

pub fn all_releases() -> Result<Vec<ReleaseVersion>> {
    let http = client()?;
    let mut found = Vec::new();

    // GitHub paginates releases. Scan pages so +MAJOR.MINOR can find older lines.
    for page in 1..=10 {
        let url = format!("{API}/repos/{REPO}/releases?per_page=100&page={page}");
        let response = http.get(url).send()?.error_for_status()?;
        let releases: Vec<Release> = response.json()?;
        if releases.is_empty() {
            break;
        }
        let count = releases.len();
        found.extend(releases.into_iter().filter_map(parse_release));
        if count < 100 {
            break;
        }
    }

    found.sort_by(|a, b| b.version.cmp(&a.version));
    found.dedup_by(|a, b| a.version == b.version);
    if found.is_empty() {
        bail!("No stable GitHub releases with the expected `{ASSET}` asset were found.");
    }
    Ok(found)
}

pub fn latest_release() -> Result<ReleaseVersion> {
    // Use GitHub's dedicated latest endpoint for the common bootstrap/update path.
    // It excludes drafts and prereleases and avoids paging through the full history.
    let release: Release = client()?
        .get(format!("{API}/repos/{REPO}/releases/latest"))
        .send()?
        .error_for_status()?
        .json()?;
    parse_release(release).ok_or_else(|| {
        anyhow::anyhow!(
            "The latest stable GitHub release did not contain the expected `{ASSET}` asset."
        )
    })
}

pub fn get_release(version: &Version) -> Result<ReleaseVersion> {
    let releases = all_releases()?;
    for release in releases {
        if &release.version == version {
            return Ok(release);
        }
    }
    bail!("Release not found for version {}", version);
}

pub fn download_and_install(release: &ReleaseVersion) -> Result<()> {
    let target_dir = version_dir(&release.version);
    let target_exe = target_dir.join(EXECUTABLE);
    if target_exe.is_file() {
        return Ok(());
    }

    let install_dir = install_dir();

    fs_err::create_dir_all(&target_dir)?;
    let temp_zip = install_dir.join(format!(".hemtt-{}.zip.part", release.version));
    let temp_dir = install_dir.join(format!(".hemtt-{}.staging", release.version));
    let _ = fs_err::remove_file(&temp_zip);
    let _ = fs_err::remove_dir_all(&temp_dir);

    eprintln!("Downloading HEMTT {}...", release.version);
    let mut response = client()?.get(&release.url).send()?.error_for_status()?;
    {
        let mut output = File::create(&temp_zip)?;
        std::io::copy(&mut response, &mut output).context("Failed to download release archive")?;
        use std::io::Write;
        output.flush()?;
        output.sync_all()?;
    }

    fs_err::create_dir_all(&temp_dir)?;
    let extraction = (|| -> Result<()> {
        let file = File::open(&temp_zip)?;
        let mut archive =
            ZipArchive::new(file).context("Downloaded release is not a valid ZIP archive")?;
        let mut found = false;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let enclosed = match entry.enclosed_name() {
                Some(p) => p.to_path_buf(),
                None => continue,
            };
            if enclosed.file_name().and_then(|n| n.to_str()) == Some(EXECUTABLE) {
                let output_path = temp_dir.join(EXECUTABLE);
                let mut output = File::create(&output_path)?;
                std::io::copy(&mut entry, &mut output)?;
                use std::io::Write;
                output.flush()?;
                output.sync_all()?;
                found = true;
                break;
            }
        }
        if !found {
            bail!("The release archive did not contain hemtt executable.");
        }
        Ok(())
    })();

    let _ = fs_err::remove_file(&temp_zip);
    if let Err(error) = extraction {
        let _ = fs_err::remove_dir_all(&temp_dir);
        return Err(error);
    }

    // Publish only after the executable has been fully downloaded and extracted.
    if target_dir.exists() {
        fs_err::remove_dir_all(&target_dir)?;
    }
    if let Err(error) = fs_err::rename(&temp_dir, &target_dir) {
        let _ = fs_err::remove_dir_all(&temp_dir);
        return Err(error).context("Could not move the downloaded version into place");
    }

    // Mark it executable on Unix-like systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs_err::metadata(target_dir.join(EXECUTABLE))?.permissions();
        perms.set_mode(0o755);
        fs_err::set_permissions(target_dir.join(EXECUTABLE), perms)?;
    }

    Ok(())
}
