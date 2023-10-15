use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
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
    pub const fn exclude(&self) -> &Vec<String> {
        &self.exclude
    }
}
