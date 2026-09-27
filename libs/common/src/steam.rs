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
