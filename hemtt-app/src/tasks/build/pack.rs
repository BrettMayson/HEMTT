use hemtt::Project;
use vfs::{PhysicalFS, SeekAndRead, VfsFileType};

use crate::{context::AddonContext, HEMTTError, Stage, Task};

pub struct Pack {}
impl Task for Pack {
    fn name(&self) -> String {
        String::from("rapify")
    }

    fn hooks(&self) -> &[Stage] {
        &[Stage::Build]
    }

    fn build(&self, ctx: &mut AddonContext) -> Result<(), HEMTTError> {
        let mut pbo = hemtt_pbo::WritablePbo::<Box<dyn SeekAndRead>>::new();
        for entry in ctx.global().fs().join(ctx.addon().source())?.walk_dir()? {
            let entry = entry?;
            if entry.filename().contains(".ht.") {
                continue;
            }
            if entry.metadata()?.file_type == VfsFileType::File {
                if entry.filename() == "config.cpp"
                    && entry.parent().unwrap().join("config.bin")?.exists()?
                {
                    ctx.debug("skipping config.cpp");
                } else {
                    ctx.debug(&format!("pack: {:?}", entry.as_str()));
                    pbo.add_file(entry.as_str(), entry.open_file()?)?;
                }
            }
        }
        let pbo_path = vfs::VfsPath::from(PhysicalFS::new(Project::find_root()?))
            .join(&ctx.addon().location().to_string())?
            .join(&ctx.addon().pbo(Some(ctx.global().project().prefix())))?;
        ctx.debug(&format!("Creating PBO at {}", pbo_path.as_str()));
        pbo.write(&mut pbo_path.create_file()?)?;
        Ok(())
    }
}
