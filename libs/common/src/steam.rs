use std::path::PathBuf;

use steamlocate::SteamDir;

#[must_use]
/// Find the path to a steam app with the given id
pub fn find_app(app_id: u32) -> Option<PathBuf> {
    let Ok(Some((app, library))) = SteamDir::locate().and_then(|s| s.find_app(app_id)) else {
        return None;
    };
    let dir = library.resolve_app_dir(&app);
    Some(dir)
}
