use std::path::PathBuf;

use crate::Error;

pub trait Resolver {
    /// Find the path to an included file
    ///
    /// # Errors
    /// if the file cannot be found
    fn find_include(&self, root: &str, from: &str, to: &str) -> Result<(PathBuf, String), Error>;
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
        fn find_include(&self, _: &str, from: &str, to: &str) -> Result<(PathBuf, String), Error> {
            let mut path = Path::new(from).parent().unwrap().to_path_buf();
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
        fn find_include(&self, _: &str, _: &str, _: &str) -> Result<(PathBuf, String), Error> {
            Err(Error::ResolveWithNoResolver)
        }
    }
}
