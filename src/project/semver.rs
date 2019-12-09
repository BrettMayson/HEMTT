use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub build: String,
}

impl SemVer {
    pub fn new(major: u32, minor: u32, patch: u32, build: String) -> Self {
        SemVer { major, minor, patch, build }
    }

    pub fn from(version: &str) -> Self {
        let mut parts = version.split('.');
        SemVer {
            major: parts.next().unwrap().parse::<u32>().unwrap(),
            minor: parts.next().unwrap().parse::<u32>().unwrap(),
            patch: parts.next().unwrap().parse::<u32>().unwrap(),
            build: if let Some(p) = parts.next() {p.to_string()} else {"".to_string()},
        }
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        if self.build.is_empty() {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        } else {
            write!(f, "{}.{}.{}.{}", self.major, self.minor, self.patch, self.build)
        }
    }
}