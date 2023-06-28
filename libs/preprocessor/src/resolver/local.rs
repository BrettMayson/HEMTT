use std::{
    io::Read,
    path::{Path, PathBuf},
};

use crate::{tokens::Token, Error};

use super::Resolver;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Default, Clone, Copy)]
/// A resolver that only follows relative paths
pub struct LocalResolver;
impl LocalResolver {}
impl Resolver for LocalResolver {
    fn find_include(
        &self,
        _: &crate::Context,
        _: &str,
        from: &str,
        to: &str,
        _source: Vec<Token>,
    ) -> Result<(PathBuf, String), Error> {
        let mut path = Path::new(from).parent().unwrap().to_path_buf();
        path.push(to);
        let mut file = std::fs::File::open(&path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok((path, content))
    }
}
