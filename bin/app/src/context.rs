use std::{
    env::temp_dir,
    path::{Path, PathBuf},
};

use hemtt_bin_project::config::Configuration;
use vfs::{AltrootFS, MemoryFS, OverlayFS, PhysicalFS, VfsPath};

use crate::{
    addons::{Addon, Location},
    error::Error,
};

pub struct Context {
    config: Configuration,
    addons: Vec<Addon>,
    fs: VfsPath,
    tmp: PathBuf,
}

impl Context {
    pub fn new(locations: &[Location]) -> Result<Self, Error> {
        let tmp = temp_dir().join("hemtt").join(
            std::env::current_dir()
                .unwrap()
                .components()
                .skip(2)
                .collect::<PathBuf>()
                .to_str()
                .unwrap()
                .replace(['\\', '/'], "_"),
        );
        Ok(Self {
            config: Configuration::from_file(Path::new("hemtt.toml"))?,
            fs: AltrootFS::new(
                OverlayFS::new(&{
                    let mut layers = vec![MemoryFS::new().into()];
                    if cfg!(target_os = "windows") {
                        layers.push(
                            AltrootFS::new(PhysicalFS::new(tmp.join("output")).into()).into(),
                        );
                    }
                    layers.push(AltrootFS::new(PhysicalFS::new(".").into()).into());
                    layers
                })
                .into(),
            )
            .into(),
            addons: Addon::scan(locations)?,
            tmp,
        })
    }

    pub const fn config(&self) -> &Configuration {
        &self.config
    }

    pub fn addons(&self) -> &[Addon] {
        &self.addons
    }

    pub const fn fs(&self) -> &VfsPath {
        &self.fs
    }

    pub const fn tmp(&self) -> &PathBuf {
        &self.tmp
    }
}

// impl Drop for Context {
//     fn drop(&mut self) {
//         remove_dir_all(self.tmp()).unwrap();
//     }
// }
