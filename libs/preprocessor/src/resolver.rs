use std::path::{Path, PathBuf};

use crate::Error;

pub trait Resolver {
    fn find_include(&self, root: &Path, from: &Path, to: &str) -> Result<(PathBuf, String), Error>;
}

pub mod resolvers {
    use std::{
        io::Read,
        path::{Path, PathBuf},
    };

    use crate::Error;

    use super::Resolver;

    pub struct LocalResolver;
    impl LocalResolver {
        #[must_use]
        pub const fn new() -> Self {
            Self
        }
    }
    impl Resolver for LocalResolver {
        fn find_include(
            &self,
            _: &Path,
            from: &Path,
            to: &str,
        ) -> Result<(PathBuf, String), Error> {
            let mut path = from.parent().unwrap().to_path_buf();
            path.push(to);
            let mut file = std::fs::File::open(&path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            Ok((path, content))
        }
    }

    pub struct NoResolver;
    impl NoResolver {
        #[must_use]
        pub const fn new() -> Self {
            Self
        }
    }
    impl Resolver for NoResolver {
        fn find_include(&self, _: &Path, _: &Path, _: &str) -> Result<(PathBuf, String), Error> {
            Err(Error::ResolveWithNoResolver)
        }
    }
}
