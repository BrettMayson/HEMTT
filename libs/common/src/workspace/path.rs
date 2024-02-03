use std::{hash::Hasher, io::Write, sync::Arc};

use tracing::trace;
use vfs::VfsPath;

use super::{Error, LayerType, Workspace};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, PartialEq, Eq)]
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

    #[must_use]
    /// Returns the workspace
    pub fn workspace(&self) -> &Workspace {
        &self.workspace
    }

    #[must_use]
    /// Is the file from an include path
    pub fn is_include(&self) -> bool {
        self.workspace
            .layers
            .iter()
            .filter(|(_, t)| *t == LayerType::Include)
            .any(|(p, _)| {
                p.join(self.path.as_str())
                    .and_then(|p| p.exists())
                    .unwrap_or(false)
            })
    }

    /// join a path to the workspace path
    ///
    /// # Errors
    /// [`Error::Vfs`] if the path could not be joined
    pub fn join(&self, path: impl AsRef<str>) -> Result<Self, Error> {
        let path = self.path.join(path)?;
        Ok(Self {
            path,
            workspace: self.workspace.clone(),
        })
    }

    /// Create a file in the workspace
    ///
    /// # Errors
    /// [`Error::Vfs`] if the file could not be created
    pub fn create_file(&self) -> Result<Box<dyn Write + Send>, Error> {
        self.path.create_file().map_err(Into::into)
    }

    /// Create a directory in the workspace
    ///
    /// # Errors
    /// [`Error::Vfs`] if the directory could not be created
    pub fn create_dir(&self) -> Result<(), Error> {
        self.path.create_dir()?;
        Ok(())
    }

    /// Check if the path exists
    ///
    /// # Errors
    /// [`Error::Vfs`] if the path could not be checked
    pub fn exists(&self) -> Result<bool, Error> {
        self.path.exists().map_err(Into::into)
    }

    /// Check if the path is a file
    ///
    /// # Errors
    /// [`Error::Vfs`] if the path could not be checked
    pub fn is_file(&self) -> Result<bool, Error> {
        self.path.is_file().map_err(Into::into)
    }

    /// Check if the path is a directory
    ///
    /// # Errors
    /// [`Error::Vfs`] if the path could not be checked
    pub fn is_dir(&self) -> Result<bool, Error> {
        self.path.is_dir().map_err(Into::into)
    }

    /// Read the path to a string
    ///
    /// # Errors
    /// [`Error::Vfs`] if the path could not be read
    pub fn read_to_string(&self) -> Result<String, Error> {
        self.path
            .read_to_string()
            .map(|s| s.replace('\r', ""))
            .map_err(Into::into)
    }

    /// Open the file for reading
    ///
    /// # Errors
    /// [`Error::Vfs`] if the file could not be opened
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

    /// Change the extension of the path
    ///
    /// # Errors
    /// [`Error::Vfs`] if the path could not be changed
    pub fn with_extension(&self, ext: &str) -> Result<Self, Error> {
        Ok(Self {
            path: {
                let current = self
                    .path
                    .filename()
                    .chars()
                    .rev()
                    .collect::<String>()
                    .split_once('.')
                    .map_or(
                        self.path.filename().as_str().chars().rev().collect(),
                        |(_, s)| s.to_string(),
                    )
                    .chars()
                    .rev()
                    .collect::<String>();
                self.path.parent().join(format!("{current}.{ext}"))?
            },
            workspace: self.workspace.clone(),
        })
    }

    /// Locate a path in the workspace
    ///
    /// Checks in order:
    /// - Relative to the current path, or absolute if the path starts with `/`
    /// - In the scanned pointers (prefix files)
    /// - In the include path
    ///
    /// # Errors
    /// [`Error::Vfs`] if the path could not be located
    pub fn locate(&self, path: &str) -> Result<Option<Self>, Error> {
        let path = path.replace('\\', "/");
        if path.starts_with('/') {
            if self.workspace.vfs.join(&path)?.exists()? {
                trace!("Located with absolute path: {:?}", path);
                return Ok(Some(Self {
                    path: self.workspace.vfs.join(path)?,
                    workspace: self.workspace.clone(),
                }));
            }
            if let Some((base, root)) = self
                .workspace
                .pointers
                .iter()
                .find(|(p, _)| path.starts_with(&format!("{p}/")))
            {
                let path = root.join(
                    path.strip_prefix(base)
                        .unwrap_or(&path)
                        .strip_prefix('/')
                        .unwrap_or(&path),
                )?;
                if path.exists()? {
                    trace!("Located with prefix pointer: {:?}", path);
                    return Ok(Some(Self {
                        path,
                        workspace: self.workspace.clone(),
                    }));
                }
            }
        }
        let path = self.path.parent().join(path)?;
        if path.exists()? {
            trace!("Located with parent: vfs {}", path.as_str());
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

    /// Read the files in a directory
    ///
    /// # Errors
    /// [`Error::Vfs`] if the directory could not be read
    pub fn read_dir(&self) -> Result<Vec<Self>, Error> {
        Ok(self
            .path
            .read_dir()?
            .map(move |p| Self {
                path: p,
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

impl std::fmt::Debug for WorkspacePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.path.as_str().fmt(f)
    }
}
