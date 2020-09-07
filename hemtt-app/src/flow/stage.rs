#[derive(Debug, PartialEq, Clone)]
pub enum Stage {
    Check,
    PreBuild,
    Build,
    PostBuild,
    Release,
    PostRelease,
    Script,
    None,
}

impl Stage {
    pub fn all() -> Vec<Self> {
        use Stage::*;
        vec![Check, PreBuild, Build, PostBuild, Release, PostRelease]
    }
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
