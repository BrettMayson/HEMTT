use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Options {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    exclude: Vec<String>,
}

impl Options {
    #[must_use]
    pub fn enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }

    #[must_use]
    #[cfg(not(target_os = "macos"))]
    pub const fn exclude(&self) -> &Vec<String> {
        &self.exclude
    }
}
