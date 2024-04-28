use std::{hash::Hasher, sync::Arc};

use hemtt_common::strip::StripInsensitive;
use vfs::{SeekAndWrite, VfsPath};

use super::{Error, LayerType, Workspace};

#[derive(Clone, PartialEq, Eq)]
pub struct WorkspacePathData {
    pub(crate) path: VfsPath,
    pub(crate) workspace: Arc<Workspace>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, PartialEq, Eq)]
/// A path in a workspace
pub struct WorkspacePath {
    pub(crate) data: Arc<WorkspacePathData>,
}

impl WorkspacePath {
    #[must_use]
    /// Returns the underlying [`VfsPath`]
    pub fn vfs(&self) -> &VfsPath {
        &self.data.path
    }

    #[must_use]
    /// Returns the workspace
    pub fn workspace(&self) -> &Workspace {
        &self.data.workspace
    }

    #[must_use]
    /// Is the file from an include path
    pub fn is_include(&self) -> bool {
        self.data
            .workspace
            .layers
            .iter()
            .filter(|(_, t)| *t == LayerType::Include)
            .any(|(p, _)| {
                p.join(self.data.path.as_str())
                    .and_then(|p| p.exists())
                    .unwrap_or(false)
            })
    }

    /// join a path to the workspace path
    ///
    /// # Errors
    /// [`Error::Vfs`] if the path could not be joined
    pub fn join(&self, path: impl AsRef<str>) -> Result<Self, Error> {
        let path = self.data.path.join(path)?;
        Ok(Self {
            data: Arc::new(WorkspacePathData {
                path,
                workspace: self.data.workspace.clone(),
            }),
        })
    }

    /// Create a file in the workspace
    ///
    /// # Errors
    /// [`Error::Vfs`] if the file could not be created
    pub fn create_file(&self) -> Result<Box<dyn SeekAndWrite + Send>, Error> {
        self.data.path.create_file().map_err(Into::into)
    }

    /// Create a directory in the workspace
    ///
    /// # Errors
    /// [`Error::Vfs`] if the directory could not be created
    pub fn create_dir(&self) -> Result<(), Error> {
        self.data.path.create_dir()?;
        Ok(())
    }

    /// Check if the path exists
    ///
    /// # Errors
    /// [`Error::Vfs`] if the path could not be checked
    pub fn exists(&self) -> Result<bool, Error> {
        self.data.path.exists().map_err(Into::into)
    }

    /// Check if the path is a file
    ///
    /// # Errors
    /// [`Error::Vfs`] if the path could not be checked
    pub fn is_file(&self) -> Result<bool, Error> {
        self.data.path.is_file().map_err(Into::into)
    }

    /// Check if the path is a directory
    ///
    /// # Errors
    /// [`Error::Vfs`] if the path could not be checked
    pub fn is_dir(&self) -> Result<bool, Error> {
        self.data.path.is_dir().map_err(Into::into)
    }

    /// Read the path to a string
    ///
    /// # Errors
    /// [`Error::Vfs`] if the path could not be read
    pub fn read_to_string(&self) -> Result<String, Error> {
        self.data
            .path
            .read_to_string()
            .map(|s| s.replace('\r', ""))
            .map_err(Into::into)
    }

    /// Open the file for reading
    ///
    /// # Errors
    /// [`Error::Vfs`] if the file could not be opened
    pub fn open_file(&self) -> Result<Box<dyn vfs::SeekAndRead + Send>, Error> {
        self.data.path.open_file().map_err(Into::into)
    }

    #[must_use]
    /// Get the path as a [`str`]
    pub fn as_str(&self) -> &str {
        self.data.path.as_str()
    }

    #[must_use]
    /// Get the parent of the path
    pub fn parent(&self) -> Self {
        Self {
            data: Arc::new(WorkspacePathData {
                path: self.data.path.parent(),
                workspace: self.data.workspace.clone(),
            }),
        }
    }

    /// Change the extension of the path
    ///
    /// # Errors
    /// [`Error::Vfs`] if the path could not be changed
    pub fn with_extension(&self, ext: &str) -> Result<Self, Error> {
        Ok(Self {
            data: Arc::new(WorkspacePathData {
                path: {
                    let current = self
                        .data
                        .path
                        .filename()
                        .chars()
                        .rev()
                        .collect::<String>()
                        .split_once('.')
                        .map_or(
                            self.data.path.filename().as_str().chars().rev().collect(),
                            |(_, s)| s.to_string(),
                        )
                        .chars()
                        .rev()
                        .collect::<String>();
                    self.data.path.parent().join(format!("{current}.{ext}"))?
                },
                workspace: self.data.workspace.clone(),
            }),
        })
    }

    /// Locate a path in the workspace
    ///
    /// Checks in order:
    /// - A3 P drive, if allowed and path starts with `/a3/`
    /// - Relative to the current path, or absolute if the path starts with `/`
    /// - In the scanned pointers (prefix files)
    /// - In the include path
    ///
    /// # Errors
    /// [`Error::Vfs`] if the path could not be located
    pub fn locate(&self, path: &str) -> Result<Option<Self>, Error> {
        let path = path.replace('\\', "/");
        let path_lower = path.to_lowercase();
        if path_lower.starts_with("/a3/") {
            if let Some(pdrive) = &self.workspace().pdrive {
                if let Some(pdrive_path) = pdrive.path_to(&path) {
                    return Ok(Some(Self {
                        data: Arc::new(WorkspacePathData {
                            path: pdrive_path,
                            workspace: self.data.workspace.clone(),
                        }),
                    }));
                }
            }
        }
        if path.starts_with('/') {
            if self.data.workspace.vfs.join(&path)?.exists()? {
                return Ok(Some(Self {
                    data: Arc::new(WorkspacePathData {
                        path: self.data.workspace.vfs.join(path)?,
                        workspace: self.data.workspace.clone(),
                    }),
                }));
            }
            if let Some((base, root)) = self
                .data
                .workspace
                .pointers
                .iter()
                .find(|(p, _)| path_lower.starts_with(&format!("{}/", p.to_lowercase())))
            {
                // Windows needs case insensitivity because p3ds are a
                // disaster. On Linux we'll be more strict to avoid
                // pain and suffering.
                let path = if cfg!(windows) {
                    root.join(
                        path_lower
                            .strip_prefix(&base.to_lowercase())
                            .unwrap_or(&path)
                            .strip_prefix('/')
                            .unwrap_or(&path),
                    )?
                } else {
                    root.join(
                        path.strip_prefix_insensitive(base)
                            .unwrap_or(&path)
                            .strip_prefix('/')
                            .unwrap_or(&path),
                    )?
                };
                if path.exists()? {
                    return Ok(Some(Self {
                        data: Arc::new(WorkspacePathData {
                            path,
                            workspace: self.data.workspace.clone(),
                        }),
                    }));
                }
            }
        }
        let path = self.data.path.parent().join(path)?;
        if path.exists()? {
            Ok(Some(Self {
                data: Arc::new(WorkspacePathData {
                    path,
                    workspace: self.data.workspace.clone(),
                }),
            }))
        } else {
            Ok(None)
        }
    }

    #[must_use]
    /// All the of addons in the workspace
    pub fn addons(&self) -> &[VfsPath] {
        &self.data.workspace.addons
    }

    #[must_use]
    /// All the of missions in the workspace
    pub fn missions(&self) -> &[VfsPath] {
        &self.data.workspace.missions
    }

    /// Walk the workspace
    ///
    /// # Errors
    /// [`Error::Vfs`] if the workspace could not be walked
    #[allow(clippy::missing_panics_doc)]
    pub fn walk_dir(&self) -> Result<Vec<Self>, Error> {
        Ok(self
            .data
            .path
            .walk_dir()?
            .filter(std::result::Result::is_ok)
            .map(|p| Self {
                data: Arc::new(WorkspacePathData {
                    path: p.expect("filtered"),
                    workspace: self.data.workspace.clone(),
                }),
            })
            .collect())
    }

    /// Read the files in a directory
    ///
    /// # Errors
    /// [`Error::Vfs`] if the directory could not be read
    pub fn read_dir(&self) -> Result<Vec<Self>, Error> {
        Ok(self
            .data
            .path
            .read_dir()?
            .map(|p| Self {
                data: Arc::new(WorkspacePathData {
                    path: p,
                    workspace: self.data.workspace.clone(),
                }),
            })
            .collect())
    }

    /// Return the metadata for the path
    ///
    /// # Errors
    /// [`Error::Vfs`] if the metadata could not be read
    pub fn metadata(&self) -> Result<vfs::VfsMetadata, Error> {
        self.data.path.metadata().map_err(Into::into)
    }

    #[must_use]
    /// Retruns the file name of the path
    pub fn filename(&self) -> String {
        self.data.path.filename()
    }

    #[must_use]
    /// Returns the extension of the path
    pub fn extension(&self) -> Option<String> {
        self.data.path.extension()
    }
}

impl std::hash::Hash for WorkspacePath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.path.as_str().hash(state);
    }
}

impl std::fmt::Display for WorkspacePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.path.as_str().fmt(f)
    }
}

impl std::fmt::Debug for WorkspacePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.path.as_str().fmt(f)
    }
}
