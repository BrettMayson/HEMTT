mod sign;
pub use sign::Sign;

use crate::{context::AddonListContext, HEMTTError, Stage, Task};

// Cleans existing files that are part of the hemtt project
#[derive(Clone)]
pub struct Release {}
impl Task for Release {
    fn name(&self) -> String {
        String::from("release")
    }

    fn hooks(&self) -> &[Stage] {
        &[Stage::PostBuild]
    }

    fn postbuild_single(&self, ctx: &mut AddonListContext) -> Result<(), HEMTTError> {
        ctx.global().rfs()?.create_dir_all().map_err(|e| e.into())
    }
}
