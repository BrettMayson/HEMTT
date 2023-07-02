use std::collections::HashMap;

use tracing::trace;
use vfs::VfsPath;

use crate::Error;

#[allow(clippy::module_name_repetitions)]
/// A resolver that can find includes
pub struct Resolver<'a> {
    vfs: &'a VfsPath,
    prefixes: HashMap<String, VfsPath>,
}

impl<'a> Resolver<'a> {
    #[must_use]
    /// Create a new resolver
    pub const fn new(vfs: &'a VfsPath, prefixes: HashMap<String, VfsPath>) -> Self {
        Self { vfs, prefixes }
    }

    /// Find an include
    ///
    /// # Errors
    ///
    /// [`Error::Vfs`] if the file fails to read
    pub fn find_include(
        &self,
        from: &VfsPath,
        to: &str,
    ) -> Result<Option<(VfsPath, String)>, Error> {
        trace!("looking for {} from {}", to, from.as_str());
        let relative = Self::relative(from, to)?;
        if relative.is_some() {
            return Ok(relative);
        }
        trace!("checking prefixes");
        let prefix = self.prefix(to)?;
        if prefix.is_some() {
            return Ok(prefix);
        }
        trace!("searching includes");
        let include = self.include(to)?;
        if include.is_some() {
            return Ok(include);
        }
        Ok(None)
    }

    fn relative(from: &VfsPath, to: &str) -> Result<Option<(VfsPath, String)>, Error> {
        let path = from.parent().join(to.replace('\\', "/"))?;
        if !path.exists()? {
            return Ok(None);
        }
        let contents = path.read_to_string()?;
        Ok(Some((path, contents)))
    }

    fn prefix(&self, to: &str) -> Result<Option<(VfsPath, String)>, Error> {
        for (prefix, path) in &self.prefixes {
            if to.starts_with(prefix) {
                let path = path.join(to.strip_prefix(prefix).unwrap().replace('\\', "/"))?;
                trace!("prefixed based file should be {}", path.as_str());
                if !path.exists()? {
                    return Ok(None);
                }
                let contents = path.read_to_string()?;
                return Ok(Some((path, contents)));
            }
        }
        Ok(None)
    }

    fn include(&self, to: &str) -> Result<Option<(VfsPath, String)>, Error> {
        let includes = self.vfs.join("include")?;
        let path = includes.join(format!(".{}", to.replace('\\', "/")))?;
        trace!("include based file should be {}", path.as_str());
        if !path.exists()? {
            return Ok(None);
        }
        let contents = path.read_to_string()?;
        Ok(Some((path, contents)))
    }
}
