use std::path::Path;

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
}

impl Context {
    pub fn new(locations: &[Location]) -> Result<Self, Error> {
        Ok(Self {
            config: Configuration::from_file(Path::new("hemtt.toml"))?,
            fs: AltrootFS::new(
                OverlayFS::new(&[
                    MemoryFS::new().into(),
                    AltrootFS::new(PhysicalFS::new(".").into()).into(),
                ])
                .into(),
            )
            .into(),
            addons: Addon::scan(locations)?,
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
}
