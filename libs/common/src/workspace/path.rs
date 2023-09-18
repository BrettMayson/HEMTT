use std::{hash::Hasher, io::Write, path::PathBuf, sync::Arc};

use vfs::VfsPath;

use super::{Error, Workspace};

#[derive(Clone, Debug, PartialEq, Eq)]
/// A path in a workspace
pub struct WorkspacePath {
    pub(crate) path: VfsPath,
    pub(crate) workspace: Arc<Workspace>,
}

impl WorkspacePath {
    #[must_use]
    /// Returns the underlying [`VfsPath`]
    pub const fn vfs(&self) -> &VfsPath {
        &self.path
    }

    /// join a path to the workspace path
    pub fn join(&self, path: impl AsRef<str>) -> Result<Self, Error> {
        let path = self.path.join(path)?;
        Ok(Self {
            path,
            workspace: self.workspace.clone(),
        })
    }

    /// Create a file in the workspace
    pub fn create_file(&self) -> Result<Box<dyn Write + Send>, Error> {
        self.path.create_file().map_err(Into::into)
    }

    /// Create a directory in the workspace
    pub fn create_dir(&self) -> Result<(), Error> {
        self.path.create_dir()?;
        Ok(())
    }

    /// Check if the path exists
    pub fn exists(&self) -> Result<bool, Error> {
        self.path.exists().map_err(Into::into)
    }

    /// Check if the path is a file
    pub fn is_file(&self) -> Result<bool, Error> {
        self.path.is_file().map_err(Into::into)
    }

    /// Check if the path is a directory
    pub fn is_dir(&self) -> Result<bool, Error> {
        self.path.is_dir().map_err(Into::into)
    }

    /// Read the path to a string
    pub fn read_to_string(&self) -> Result<String, Error> {
        self.path.read_to_string().map_err(Into::into)
    }

    /// Open the file for reading
    pub fn open_file(&self) -> Result<Box<dyn vfs::SeekAndRead + Send>, Error> {
        self.path.open_file().map_err(Into::into)
    }

    #[must_use]
    /// Get the path as a [`str`]
    pub fn as_str(&self) -> &str {
        self.path.as_str()
    }

    #[must_use]
    /// Get the parent of the path
    pub fn parent(&self) -> Self {
        Self {
            path: self.path.parent(),
            workspace: self.workspace.clone(),
        }
    }

    /// Locate a path in the workspace
    ///
    /// Checks in order:
    /// - Relative to the current path, or absolute if the path starts with `/`
    /// - In the scanned pointers (prefix files)
    /// - In the include path
    pub fn locate(&self, path: &str) -> Result<Option<Self>, Error> {
        let path = path.replace('\\', "/");
        if path.starts_with('/') {
            if self.workspace.vfs.join(&path)?.exists()? {
                return Ok(Some(Self {
                    path: self.workspace.vfs.join(path)?,
                    workspace: self.workspace.clone(),
                }));
            }
            if let Some(path) = self.workspace.pointers.get(&PathBuf::from(path.as_str())) {
                return Ok(Some(Self {
                    path: path.clone(),
                    workspace: self.workspace.clone(),
                }));
            }
        }
        let path = self.path.parent().join(path)?;
        if path.exists()? {
            Ok(Some(Self {
                path,
                workspace: self.workspace.clone(),
            }))
        } else {
            Ok(None)
        }
    }

    #[must_use]
    /// All the of addons in the workspace
    pub fn addons(&self) -> &[VfsPath] {
        &self.workspace.addons
    }

    #[must_use]
    /// All the of missions in the workspace
    pub fn missions(&self) -> &[VfsPath] {
        &self.workspace.missions
    }

    /// Walk the workspace
    ///
    /// # Errors
    /// [`Error::Vfs`] if the workspace could not be walked
    #[allow(clippy::missing_panics_doc)]
    pub fn walk_dir(&self) -> Result<Vec<Self>, Error> {
        Ok(self
            .path
            .walk_dir()?
            .filter(std::result::Result::is_ok)
            .map(move |p| Self {
                path: p.expect("filtered"),
                workspace: self.workspace.clone(),
            })
            .collect())
    }

    /// Return the metadata for the path
    ///
    /// # Errors
    /// [`Error::Vfs`] if the metadata could not be read
    pub fn metadata(&self) -> Result<vfs::VfsMetadata, Error> {
        self.path.metadata().map_err(Into::into)
    }

    #[must_use]
    /// Retruns the file name of the path
    pub fn filename(&self) -> String {
        self.path.filename()
    }

    #[must_use]
    /// Returns the extension of the path
    pub fn extension(&self) -> Option<String> {
        self.path.extension()
    }
}

impl std::hash::Hash for WorkspacePath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.as_str().hash(state);
    }
}

impl std::fmt::Display for WorkspacePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.path.as_str().fmt(f)
    }
}
