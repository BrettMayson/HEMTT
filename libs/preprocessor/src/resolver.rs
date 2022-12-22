use std::path::PathBuf;

use hemtt_tokens::Token;

use crate::{Context, Error};

/// A trait for resolving includes
pub trait Resolver {
    /// Find the path to an included file
    ///
    /// # Errors
    /// [`Error::IncludeNotFound`] if the file is not found
    fn find_include(
        &self,
        context: &Context,
        root: &str,
        from: &str,
        to: &str,
        source: Vec<Token>,
    ) -> Result<(PathBuf, String), Error>;
}

/// Built-in include resolvers
pub mod resolvers {
    use std::{
        io::Read,
        path::{Path, PathBuf},
    };

    use hemtt_tokens::Token;

    use crate::Error;

    use super::Resolver;

    /// A resolver that only follows relative paths
    pub struct LocalResolver;
    impl LocalResolver {
        #[must_use]
        /// Create a new `LocalResolver`
        pub const fn new() -> Self {
            Self
        }
    }
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

    /// A resolver that does not resolve includes
    pub struct NoResolver;
    impl NoResolver {
        #[must_use]
        /// Create a new `NoResolver`
        pub const fn new() -> Self {
            Self
        }
    }
    impl Resolver for NoResolver {
        fn find_include(
            &self,
            context: &crate::Context,
            _: &str,
            _: &str,
            _: &str,
            source: Vec<Token>,
        ) -> Result<(PathBuf, String), Error> {
            Err(Error::ResolveWithNoResolver {
                token: Box::new(source.first().unwrap().clone()),
                trace: context.trace(),
            })
        }
    }
}
