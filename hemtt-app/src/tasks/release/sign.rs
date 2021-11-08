use std::{convert::TryInto, sync::RwLock};

use hemtt_pbo::sync::ReadablePbo;
use hemtt_sign::BIPrivateKey;

use crate::{
    context::{AddonContext, AddonListContext},
    HEMTTError, Stage, Task,
};

// Cleans existing files that are part of the hemtt project
#[derive(Debug, Default)]
pub struct Sign {
    key: RwLock<Option<BIPrivateKey>>,
}
impl Task for Sign {
    fn name(&self) -> String {
        String::from("sign")
    }

    fn hooks(&self) -> &[Stage] {
        &[Stage::PreRelease, Stage::Release]
    }

    fn prerelease_single(&self, ctx: &mut AddonListContext) -> Result<(), HEMTTError> {
        let key = BIPrivateKey::generate(1024, ctx.global().project().authority()?);
        key.write(&mut {
            let keys = ctx.global().rfs()?.join("keys")?;
            keys.create_dir_all()?;
            keys.join(&ctx.global().project().key_name()?)?
                .create_file()?
        })?;
        *self.key.write().unwrap() = Some(key);
        Ok(())
    }

    fn release(&self, ctx: &mut AddonContext) -> Result<(), HEMTTError> {
        let project = ctx.global().project();
        self.key
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .sign(
                &mut ReadablePbo::from(
                    ctx.global()
                        .vfs()
                        .join(&ctx.addon().location_pbo(Some(project.prefix())))?
                        .open_file()?,
                )?,
                project.sig_version().try_into().unwrap(),
            )
            .write({
                let path = ctx.global().rfs()?.join(
                    &ctx.addon()
                        .location_sig(Some(project.prefix()), &project.authority()?),
                )?;
                path.parent().unwrap().create_dir_all()?;
                &mut path.create_file()?
            })?;
        Ok(())
    }
}
