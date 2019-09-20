#[derive(Debug, PartialEq, Clone)]
pub enum Stage {
    Check,
    PreBuild,
    Build,
    PostBuild,
    ReleaseBuild,
    Script,
    None,
}
impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
