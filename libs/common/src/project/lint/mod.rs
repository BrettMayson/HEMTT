use serde::{Deserialize, Serialize};

mod sqf;

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Options {
    #[serde(default)]
    sqf: sqf::Options,
}

impl Options {
    #[must_use]
    pub const fn sqf(&self) -> &sqf::Options {
        &self.sqf
    }
}
