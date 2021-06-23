#[derive(Debug, PartialEq, Clone)]
pub enum Stage {
    /// Quick tasks to verify the project
    Check,
    /// Before the files are built into a PBO
    PreBuild,
    /// Building the PBO
    Build,
    /// After the PBOs are built
    PostBuild,
    /// Before HEMTT creates the release
    PreRelease,
    /// Creating the release folder
    Release,
    /// After the release folder is created
    PostRelease,
}

impl Stage {
    pub fn check() -> Vec<Self> {
        use Stage::*;
        vec![Check]
    }

    pub fn standard() -> Vec<Self> {
        use Stage::*;
        vec![Check, PreBuild, Build, PostBuild]
    }

    pub fn release() -> Vec<Self> {
        use Stage::*;
        vec![
            Check,
            PreBuild,
            Build,
            PostBuild,
            PreRelease,
            Release,
            PostRelease,
        ]
    }
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
