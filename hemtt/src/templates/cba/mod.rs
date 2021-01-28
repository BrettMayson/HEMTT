use std::io::Write;
use std::path::PathBuf;

use vfs::FileSystem;

use crate::{self as hemtt, Project};
use crate::{Addon, HEMTTError, Template};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct CBA {
    path: PathBuf,
}

impl CBA {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path<P: Into<PathBuf>>(&self, path: P) -> PathBuf {
        let mut new = self.path.clone();
        new.push(path.into());
        new
    }
}

impl Template for CBA {
    fn init(&self) -> Result<(), HEMTTError> {
        if !self.path.exists() {
            std::fs::create_dir_all(&self.path)?;
        }
        for file in InitAssets::iter() {
            let mut f = create_file!({
                let mut path = self.path.clone();
                path.push(file.as_ref());
                trace!("Writing init file: {:?}", path);
                std::fs::create_dir_all(path.parent().unwrap())?;
                path
            })?;
            f.write_all(&InitAssets::get(file.as_ref()).unwrap())?;
        }
        Ok(())
    }
    fn new_addon(&self, addon: &Addon) -> Result<(), HEMTTError> {
        let source = addon.source();
        if !self.path.join(source).exists() {
            std::fs::create_dir_all(&source)?;
        }
        for file in AddonAssets::iter() {
            let mut f = create_file!({
                let mut path = self.path.clone();
                path.push(source);
                path.push(file.as_ref());
                trace!("Writing addon file: {:?}", path);
                std::fs::create_dir_all(path.parent().unwrap())?;
                path
            })?;
            let content = AddonAssets::get(file.as_ref()).unwrap();
            f.write_all(
                super::replace(
                    &super::Vars {
                        addon: &addon.name(),
                    },
                    String::from_utf8(content.to_vec()).unwrap(),
                )
                .as_bytes(),
            )?;
        }
        Ok(())
    }
    fn new_function(&self, addon: &Addon, name: &str) -> Result<PathBuf, HEMTTError> {
        let function_file = {
            let mut path = self.path(addon.source());
            path.push("functions");
            path.push(format!("fnc_{}.sqf", name));
            path
        };
        if function_file.exists() {
            return Err(HEMTTError::User("The function already exists".to_string()));
        }
        trace!("function file: {:?}", function_file);
        let mut f = create_file!(&function_file)?;
        f.write_all(b"#include \"script_component.hpp\"\n")?;
        f.flush()?;
        let mut f = std::fs::OpenOptions::new().write(true).append(true).open({
            let mut path = self.path(addon.source());
            path.push("XEH_PREP.hpp");
            path
        })?;
        f.write_all(format!("PREP({});\n", name).as_bytes())?;
        f.flush()?;
        Ok(function_file)
    }
}

#[derive(rust_embed::RustEmbed)]
#[folder = "src/templates/cba/init/"]
struct InitAssets;

#[derive(rust_embed::RustEmbed)]
#[folder = "src/templates/cba/addon/"]
struct AddonAssets;

#[cfg(test)]
mod test {
    use super::Template;
    use crate::{Addon, AddonLocation};
    #[test]
    fn init() {
        let folder = {
            let mut tmp = std::env::temp_dir();
            tmp.push(uuid::Uuid::new_v4().to_string());
            tmp
        };
        let template = super::CBA::new(folder.clone());
        template.init().unwrap();
        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn addon() {
        let folder = {
            let mut tmp = std::env::temp_dir();
            tmp.push(uuid::Uuid::new_v4().to_string());
            tmp
        };
        let template = super::CBA::new(folder.clone());
        template.init().unwrap();
        template
            .new_addon(&Addon::new("test", AddonLocation::Addons).unwrap())
            .unwrap();
        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn function() {
        let folder = {
            let mut tmp = std::env::temp_dir();
            tmp.push(uuid::Uuid::new_v4().to_string());
            tmp
        };
        let template = super::CBA::new(folder.clone());
        template.init().unwrap();
        template
            .new_addon(&Addon::new("test", AddonLocation::Addons).unwrap())
            .unwrap();
        template
            .new_function(&Addon::new("test", AddonLocation::Addons).unwrap(), "test")
            .unwrap();
        std::fs::remove_dir_all(folder).unwrap();
    }
}
