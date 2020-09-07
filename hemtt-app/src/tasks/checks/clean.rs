use crate::{context::AddonContext, HEMTTError, OkSkip, Stage, Task};

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

    fn check(&self, context: &mut AddonContext) -> Result<OkSkip, HEMTTError> {
        let target = context.addon.destination(
            &hemtt::Project::find_root()?,
            Some(&context.global.project.prefix),
            None,
        );
        if target.exists() {
            remove_file!(target)?;
        }
        Ok((true, false))
    }
}
