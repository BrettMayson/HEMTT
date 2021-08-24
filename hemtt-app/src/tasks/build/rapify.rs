use std::path::PathBuf;

use vfs::VfsFileType;

use crate::{context::AddonContext, HEMTTError, Stage, Task};

pub fn can_rapify(path: &str) -> bool {
    let path = PathBuf::from(path);
    let name = path
        .extension()
        .unwrap_or_else(|| std::ffi::OsStr::new(""))
        .to_str()
        .unwrap();
    ["cpp", "rvmat", "ext"].contains(&name)
}

pub struct Rapify {}
impl Task for Rapify {
    fn name(&self) -> String {
        String::from("rapify")
    }

    fn hooks(&self) -> &[Stage] {
        &[Stage::Build]
    }

    fn build(&self, ctx: &mut AddonContext) -> Result<(), HEMTTError> {
        for entry in ctx.global().vfs().join(ctx.addon().source())?.walk_dir()? {
            let entry = entry?;
            if entry.metadata()?.file_type == VfsFileType::File && can_rapify(entry.as_str()) {
                ctx.debug(&format!("rapify: {:?}", entry.as_str()));
                let mut buf = String::new();
                entry.open_file()?.read_to_string(&mut buf)?;
                let simplified =
                    hemtt_arma_config::simplify::Config::from_ast(hemtt_arma_config::parse(&buf, entry.as_str())?)
                        .unwrap();
                let mut out = if entry.filename() == "config.cpp" {
                    entry.parent().unwrap().join("config.bin")?
                } else {
                    entry
                }
                .create_file()?;
                simplified.write_rapified(&mut out).unwrap();
            }
        }
        Ok(())
    }
}
