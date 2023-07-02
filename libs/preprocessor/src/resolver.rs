use std::collections::HashMap;

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
    pub fn find_include(
        &self,
        from: &VfsPath,
        to: &str,
    ) -> Result<Option<(VfsPath, String)>, Error> {
        let relative = Self::relative(from, to)?;
        if relative.is_some() {
            return Ok(relative);
        }
        let prefix = self.prefix(to)?;
        if prefix.is_some() {
            return Ok(prefix);
        }
        let include = self.include(to)?;
        if include.is_some() {
            return Ok(include);
        }
        Ok(None)
    }

    fn relative(from: &VfsPath, to: &str) -> Result<Option<(VfsPath, String)>, Error> {
        let path = from.parent().join(to)?;
        if !path.exists()? {
            return Ok(None);
        }
        let contents = path.read_to_string()?;
        Ok(Some((path, contents)))
    }

    fn prefix(&self, to: &str) -> Result<Option<(VfsPath, String)>, Error> {
        for (prefix, path) in &self.prefixes {
            if to.starts_with(prefix) {
                let path = path.join(to)?;
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
        let includes = self.vfs.join("includes")?;
        let path = includes.join(to)?;
        if !path.exists()? {
            return Ok(None);
        }
        let contents = path.read_to_string()?;
        Ok(Some((path, contents)))
    }
}
