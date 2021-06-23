use std::convert::TryInto;

use hemtt_pbo::ReadablePbo;
use hemtt_sign::BIPrivateKey;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{context::AddonListContext, HEMTTError, Stage, Task};

// Cleans existing files that are part of the hemtt project
#[derive(Clone)]
pub struct Sign {}
impl Task for Sign {
    fn name(&self) -> String {
        String::from("sign")
    }

    fn hooks(&self) -> &[Stage] {
        &[Stage::PreRelease]
    }

    fn prerelease_single(&self, ctx: &mut AddonListContext) -> Result<(), HEMTTError> {
        let private_key = BIPrivateKey::generate(1024, ctx.global().project().authority()?);
        ctx.addons()
            .par_iter()
            .map(|ctx| {
                let project = ctx.global().project();
                private_key
                    .sign(
                        &mut ReadablePbo::from(
                            ctx.global()
                                .vfs()
                                .join(&ctx.addon().location_pbo(Some(project.prefix())))?
                                .open_file()?,
                        )?,
                        project.sig_version().try_into().unwrap(),
                    )
                    .write(
                        {
                            let path = ctx
                            .global()
                            .rfs()?
                            .join(
                                &ctx.addon()
                                    .location_sig(Some(project.prefix()), &project.authority()?),
                            )?;
                            path.parent().unwrap().create_dir_all()?;
                            &mut path.create_file()?
                        }
                    )?;
                Ok(())
            })
            .collect::<Result<(), HEMTTError>>()?;
        Ok(())
    }
}
