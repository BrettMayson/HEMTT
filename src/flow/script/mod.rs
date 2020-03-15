use serde::{Deserialize, Serialize};

mod task;
pub use task::Script;

#[derive(Serialize, Deserialize, Clone)]
pub struct BuildScript {
    #[serde(default = "default_only_development")]
    pub only_development: bool,

    #[serde(default = "default_only_release")]
    pub only_release: bool,

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
impl BuildScript {
    pub fn should_run(&self, release: bool) -> bool {
        (!self.only_development && !self.only_release) ||
            (self.only_development && !release) ||
            (self.only_release && release)
    }
}

fn default_only_development() -> bool {
    false
}

fn default_only_release() -> bool {
    false
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
