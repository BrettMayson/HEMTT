pub mod local;
pub mod none;

use std::path::PathBuf;

use crate::{tokens::Token, Context, Error};

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
