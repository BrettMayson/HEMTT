use std::io::Read;
use std::io::Write;

use crate::error::Error;

/// Check if there is a newer version of HEMTT available
///
/// # Errors
/// [`Error::Update`] if the update check failed
pub fn check() -> Result<Option<String>, Error> {
    if env!("HEMTT_VERSION").contains("-local") || env!("HEMTT_VERSION").contains("-debug") {
        debug!("skip update check for local / debug version");
        return Ok(None);
    }
    let tmp_folder = std::env::temp_dir().join("hemtt");
    if !tmp_folder.exists() {
        fs_err::create_dir_all(&tmp_folder)?;
    }
    let tmp_latest = tmp_folder.join("latest");
    let need_check = if tmp_latest.exists() {
        // only check if the file is older than 12 hours
        let metadata = fs_err::metadata(&tmp_latest)?;
        let modified = metadata.modified()?;
        let now = std::time::SystemTime::now();
        let duration = now.duration_since(modified).unwrap_or_default();
        duration.as_secs() > 12 * 60 * 60
    } else {
        true
    };
    if need_check {
        let Ok(client) = reqwest::blocking::Client::builder()
            .user_agent("HEMTT")
            .build()
        else {
            return Err(Error::Update(String::from("Failed to create HTTP client")));
        };
        let Ok(response) = client
            .get("https://api.github.com/repos/brettmayson/HEMTT/releases/latest")
            .send()
        else {
            return Err(Error::Update(String::from(
                "Failed to get latest release from GitHub",
            )));
        };
        let Ok(release): reqwest::Result<Release> = response.json() else {
            return Err(Error::Update(String::from(
                "Failed to parse latest release from GitHub",
            )));
        };
        let mut file = fs_err::File::create(&tmp_latest)?;
        file.write_all(release.tag_name.as_bytes())?;
    }
    let current = env!("HEMTT_VERSION");
    let Ok(current) = semver::Version::parse(current) else {
        return Err(Error::Update(String::from(
            "Failed to parse current version",
        )));
    };
    let mut file = fs_err::File::open(&tmp_latest)?;
    let mut latest = String::new();
    file.read_to_string(&mut latest)?;
    let Ok(latest) = semver::Version::parse(&latest[1..]) else {
        return Err(Error::Update(String::from(
            "Failed to parse latest version",
        )));
    };
    if latest > current {
        Ok(Some(latest.to_string()))
    } else {
        Ok(None)
    }
}

use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
struct Release {
    pub tag_name: String,
}
