use crate::{
    context::{AddonContext, AddonListContext},
    HEMTTError, OkSkip, Stage, Task,
};

pub struct Preprocess {}

impl Task for Preprocess {
    fn name(&self) -> String {
        String::from("preprocess")
    }

    fn hooks(&self) -> &[Stage] {
        &[Stage::Check, Stage::PreBuild, Stage::PostBuild]
    }

    fn prebuild(&self, ctx: &mut AddonContext) -> Result<OkSkip, HEMTTError> {
        todo!()
    }
}
