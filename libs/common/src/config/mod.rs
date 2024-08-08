#![allow(clippy::module_name_repetitions)]

//! Read the project configuration into a [`ProjectConfig`] struct

use std::path::Path;

use tracing::warn;

mod addon;
mod pdrive;
mod project;

pub use addon::AddonConfig;
pub use pdrive::PDriveOption;
pub use project::{hemtt::launch::LaunchOptions, ProjectConfig};

fn deprecated(file: &Path, key: &str, replacement: &str, info: Option<&str>) {
    warn!(
        "Use of deprecated key '{}' in '{}'. Use '{}' instead.{}",
        key,
        file.display(),
        replacement,
        info.map(|i| format!("\n  {}", i)).unwrap_or_default()
    );
}
