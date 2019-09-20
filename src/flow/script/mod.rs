use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct BuildScript {
    #[serde(default = "default_debug")]
    pub debug: bool,
    #[serde(default = "default_release")]
    pub release: bool,
    #[serde(default = "default_foreach")]
    pub foreach: bool,
    #[serde(default = "default_parallel")]
    pub parallel: bool,
    #[serde(default = "default_show_output")]
    pub show_output: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub steps: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub steps_windows: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub steps_linux: Vec<String>,
}

fn default_debug() -> bool {
    true
}

fn default_release() -> bool {
    true
}

fn default_foreach() -> bool {
    false
}

fn default_parallel() -> bool {
    false
}

fn default_show_output() -> bool {
    false
}
