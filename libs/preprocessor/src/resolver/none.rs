use std::path::PathBuf;

use crate::{tokens::Token, Error};

use super::Resolver;

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
