use std::{io::Write, path::PathBuf, sync::Arc};

use vfs::VfsPath;

use super::{Error, Workspace};

#[derive(Clone)]
pub struct WorkspacePath {
    pub(crate) path: VfsPath,
    pub(crate) workspace: Arc<Workspace>,
}

impl WorkspacePath {
    pub fn join(&self, path: &str) -> Result<Self, Error> {
        let path = self.path.join(path)?;
        Ok(Self {
            path,
            workspace: self.workspace.clone(),
        })
    }

    pub fn create_file(&self) -> Result<Box<dyn Write + Send>, Error> {
        self.path.create_file().map_err(Into::into)
    }

    pub fn create_dir(&self) -> Result<(), Error> {
        self.path.create_dir()?;
        Ok(())
    }

    pub fn exists(&self) -> Result<bool, Error> {
        self.path.exists().map_err(Into::into)
    }

    pub fn is_file(&self) -> Result<bool, Error> {
        self.path.is_file().map_err(Into::into)
    }

    pub fn is_dir(&self) -> Result<bool, Error> {
        self.path.is_dir().map_err(Into::into)
    }

    pub fn read_to_string(&self) -> Result<String, Error> {
        self.path.read_to_string().map_err(Into::into)
    }

    pub fn as_str(&self) -> &str {
        self.path.as_str()
    }

    pub fn locate(&self, path: &str) -> Result<Option<Self>, Error> {
        println!("locate `{}` from `{}`", path, self.path.parent().as_str());
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

    pub fn addons(&self) -> &[VfsPath] {
        &self.workspace.addons
    }

    pub fn missions(&self) -> &[VfsPath] {
        &self.workspace.missions
    }
}
