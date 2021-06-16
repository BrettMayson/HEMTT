use std::fs::read_to_string;

use hemtt::HEMTTError;

pub struct ResolvedFile {
    path: String,
    data: String,
}
impl ResolvedFile {
    pub fn new<S1, S2>(path: S1, data: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            path: path.into(),
            data: data.into(),
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn data(&self) -> &str {
        &self.data
    }
}

pub trait Resolver: Clone {
    fn resolve(&self, root: &str, from: &str, to: &str) -> Result<ResolvedFile, HEMTTError>;
}

#[derive(Clone, Copy)]
pub struct Basic;
impl Resolver for Basic {
    fn resolve(&self, root: &str, _: &str, to: &str) -> Result<ResolvedFile, HEMTTError> {
        let path = format!("{}/{}", root, to);
        Ok(ResolvedFile::new(&path, &read_to_string(&path)?))
    }
}
