mod sign;
pub use sign::Sign;

use crate::{
    context::{AddonContext, AddonListContext},
    HEMTTError, Stage, Task,
};

// Cleans existing files that are part of the hemtt project
#[derive(Clone)]
pub struct Release {}
impl Task for Release {
    fn name(&self) -> String {
        String::from("release")
    }

    fn hooks(&self) -> &[Stage] {
        &[Stage::Check, Stage::PostBuild, Stage::Release]
    }

    fn check_single(&self, ctx: &mut AddonListContext) -> Result<(), HEMTTError> {
        if ctx.global().release_path().exists() {
            Err(HEMTTError::ReleaseExists(ctx.global().release_path().clone()))
        } else {
            Ok(())
        }
    }

    fn postbuild_single(&self, ctx: &mut AddonListContext) -> Result<(), HEMTTError> {
        ctx.global().rfs()?.create_dir_all().map_err(|e| e.into())
    }

    fn release(&self, ctx: &mut AddonContext) -> Result<(), HEMTTError> {
        ctx.global()
            .pfs()
            .join(
                &ctx.addon()
                    .location_pbo(Some(ctx.global().project().prefix())),
            )?
            .copy_file(&{
                let dst = ctx.global().rfs()?.join(
                    &ctx.addon()
                        .location_pbo(Some(ctx.global().project().prefix())),
                )?;
                dst.parent().unwrap().create_dir_all()?;
                dst
            })
            .map_err(|e| e.into())
    }
}
