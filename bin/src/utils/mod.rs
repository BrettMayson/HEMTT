pub mod audio;
pub mod bom;
pub mod config;
pub mod inspect;
pub mod p3d;
pub mod paa;
pub mod pbo;
pub mod photoshoot;
pub mod sqf;
pub mod verify;

#[must_use]
#[allow(clippy::while_float)]
#[allow(clippy::cast_precision_loss)]
/// Convert bytes to human readable format
pub fn bytes_to_human_readable(bytes: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 {
        size /= 1024.0;
        unit += 1;
    }
    format!("{:.2} {}", size, units[unit])
}
