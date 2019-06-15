use std::path::PathBuf;

use crate::build::BuildResult;

pub struct State<'a> {
    pub stage: Stage,
    pub result: Option<&'a BuildResult>,
    pub addons: Vec<PathBuf>,
}
impl<'a> State<'a> {
    pub fn new(addons: &[PathBuf]) -> State {
        State {
            stage: Stage::PreBuild,
            result: None,
            addons: addons.to_vec(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Stage {
    PreBuild,
    PostBuild,
    ReleaseBuild,
    Script,
}
impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
