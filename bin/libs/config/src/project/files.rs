use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Options {
    #[serde(default)]
    /// Files to be included in the output folder, supports glob patterns
    include: Vec<String>,
    #[serde(default)]
    /// Files to be excluded from being included in PBO files, supports glob patterns
    exclude: Vec<String>,
}

impl Options {
    #[must_use]
    pub fn include(&self) -> Vec<String> {
        let mut files = self.include.clone();
        for default in [
            "mod.cpp",
            "meta.cpp",
            "LICENSE",
            "logo_ca.paa",
            "logo_co.paa",
        ]
        .iter()
        .map(std::string::ToString::to_string)
        {
            if !files.contains(&default) {
                files.push(default.clone());
            }
        }
        files.sort();
        files.dedup();
        files
    }

    #[must_use]
    pub const fn exclude(&self) -> &Vec<String> {
        &self.exclude
    }
}
