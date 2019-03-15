use std::path::PathBuf;

pub struct BuildResult {
    pub built: Vec<PBOResult>,
    pub skipped: Vec<PBOResult>,
    pub failed: Vec<PBOResult>,
}

impl BuildResult {
    pub fn new() -> Self {
        BuildResult {
            built: Vec::new(),
            skipped: Vec::new(),
            failed: Vec::new(),
        }
    }
}

pub struct PBOResult {
    pub source: PathBuf,
    pub target: PathBuf,
    pub time: u128,
}
impl PBOResult {
    pub fn new(source: PathBuf, target: PathBuf, time: u128) -> PBOResult {
        PBOResult{
            source: source,
            target: target,
            time: time,
        }
    }
}
impl std::fmt::Display for PBOResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.source.file_name().unwrap().to_str().unwrap())
    }
}
impl std::fmt::Debug for PBOResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.source.file_name().unwrap().to_str().unwrap())
    }
}
