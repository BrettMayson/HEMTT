use std::io::Write;
use std::path::PathBuf;

use crate as hemtt;
use crate::{Addon, HEMTTError, Template};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct CBA {
    path: PathBuf,
}

impl CBA {
    pub const fn new(path: PathBuf) -> Self {
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
            f.write_all(&InitAssets::get(file.as_ref()).unwrap().data)?;
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
                        addon: addon.name(),
                    },
                    String::from_utf8(content.data.to_vec()).unwrap(),
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
    use std::path::PathBuf;

    use super::Template;
    use crate::{Addon, AddonLocation};

    struct TempFolder(PathBuf);
    impl TempFolder {
        fn new() -> Self {
            let mut tmp = std::env::temp_dir();
            tmp.push(uuid::Uuid::new_v4().to_string());
            Self(tmp)
        }
        fn path(&self) -> PathBuf {
            self.0.clone()
        }
    }
    impl Drop for TempFolder {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn init() {
        let tmp = TempFolder::new();
        super::super::init(super::super::Templates::CBA, tmp.path()).unwrap();
    }

    #[test]
    fn addon() {
        let tmp = TempFolder::new();
        let template = super::CBA::new(tmp.path());
        template.init().unwrap();
        template
            .new_addon(&Addon::new("test", AddonLocation::Addons).unwrap())
            .unwrap();
    }

    #[test]
    fn function() {
        let tmp = TempFolder::new();
        let template = super::CBA::new(tmp.path());
        template.init().unwrap();
        template
            .new_addon(&Addon::new("test", AddonLocation::Addons).unwrap())
            .unwrap();
        template
            .new_function(&Addon::new("test", AddonLocation::Addons).unwrap(), "test")
            .unwrap();
    }

    #[test]
    fn function_collision() {
        let tmp = TempFolder::new();
        let template = super::CBA::new(tmp.path());
        template.init().unwrap();
        template
            .new_addon(&Addon::new("test", AddonLocation::Addons).unwrap())
            .unwrap();
        template
            .new_function(&Addon::new("test", AddonLocation::Addons).unwrap(), "test")
            .unwrap();
        assert!(template
            .new_function(&Addon::new("test", AddonLocation::Addons).unwrap(), "test")
            .is_err());
    }
}
