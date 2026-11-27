use std::path::PathBuf;

#[must_use]
/// Find the path to a steam app with the given id
///
/// Searches all detected Steam installations (native, Flatpak, Snap, etc.) so that a valid
/// installation is found even when an invalid or empty Steam directory is detected first.
pub fn find_app(app_id: u32) -> Option<PathBuf> {
    let steam_dirs = steamlocate::locate_all().ok()?;
    for steam_dir in steam_dirs {
        if let Ok(Some((app, library))) = steam_dir.find_app(app_id) {
            return Some(library.resolve_app_dir(&app));
        }
    }
    None
}

/// The environment variable that overrides where Arma 3 is installed
pub const ARMA3_PATH_VAR: &str = "HEMTT_ARMA3_PATH";

#[must_use]
/// Find the Arma 3 installation
///
/// `HEMTT_ARMA3_PATH` is used if it is set, for installs Steam doesn't know
/// about, and so tests never touch a real installation. Otherwise Steam is searched.
pub fn find_arma3() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os(ARMA3_PATH_VAR).filter(|p| !p.is_empty()) {
        return Some(PathBuf::from(path));
    }
    find_app(107_410)
}
