use crate::{context::AddonContext, HEMTTError, Stage, Task};

// Cleans existing files that are part of the hemtt project
#[derive(Clone)]
pub struct Clean {}
impl Task for Clean {
    fn name(&self) -> String {
        String::from("clean")
    }

    fn hooks(&self) -> &[Stage] {
        &[Stage::Check]
    }

    fn check(&self, ctx: &mut AddonContext) -> Result<(), HEMTTError> {
        let target = ctx.addon().destination(
            &hemtt::Project::find_root()?,
            Some(ctx.global().project().prefix()),
            None,
        );
        if target.exists() {
            remove_file!(target)?;
        }
        Ok(())
    }
}
