#![allow(clippy::module_name_repetitions)]

//! Read the project configuration into a [`ProjectConfig`] struct

use tracing::warn;

mod addon;
mod pdrive;
mod project;

pub use addon::AddonConfig;
pub use pdrive::PDriveOption;
pub use project::{
    hemtt::launch::LaunchOptions,
    lint::{LintConfig, LintConfigOverride, LintEnabled},
    ProjectConfig,
};

fn deprecated(file: &str, key: &str, replacement: &str, info: Option<&str>) {
    warn!(
        "Use of deprecated key '{}' in '{}'. Use '{}' instead.{}",
        key,
        file,
        replacement,
        info.map(|i| format!("\n  {i}")).unwrap_or_default()
    );
}
