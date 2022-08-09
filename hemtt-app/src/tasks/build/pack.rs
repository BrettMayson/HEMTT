use vfs::{SeekAndRead, VfsFileType};

use crate::{context::AddonContext, HEMTTError, Stage, Task};

pub struct Pack {}
impl Task for Pack {
    fn name(&self) -> String {
        String::from("pack")
    }

    fn hooks(&self) -> &[Stage] {
        &[Stage::Build]
    }

    fn build(&self, ctx: &mut AddonContext) -> Result<(), HEMTTError> {
        if ctx.skip() {
            return Ok(());
        }

        let mut pbo = hemtt_pbo::WritablePbo::<Box<dyn SeekAndRead>>::new();

        pbo.add_extension("prefix", ctx.prefix());
        pbo.add_extension("hemtt", *crate::VERSION);
        pbo.add_extension("version", &ctx.global().project().version().to_string());

        for entry in ctx.global().vfs().join(ctx.addon().source())?.walk_dir()? {
            let entry = entry?;
            if entry.filename().contains(".ht.") || entry.filename().starts_with('$') {
                continue;
            }
            if entry.metadata()?.file_type == VfsFileType::File {
                if entry.filename() == "config.cpp"
                    && entry.parent().unwrap().join("config.bin")?.exists()?
                {
                    ctx.debug("skipping config.cpp");
                } else {
                    ctx.debug(&format!(
                        "pack: {:?} from {:?}",
                        entry.as_str(),
                        ctx.addon().source()
                    ));
                    pbo.add_file(
                        entry
                            .as_str()
                            .trim_start_matches(&format!("/{}/", ctx.addon().source())),
                        entry.open_file()?,
                    )?;
                }
            }
        }
        let pbo_path = ctx.global().pfs().join(
            &ctx.addon()
                .location_pbo(Some(ctx.global().project().prefix())),
        )?;
        ctx.debug(&format!("Creating PBO at {}", pbo_path.as_str()));
        pbo.write(&mut pbo_path.create_file()?, true)?;
        Ok(())
    }
}
