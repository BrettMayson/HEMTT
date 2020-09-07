use crate::{
    context::{AddonContext, AddonListContext},
    HEMTTError, OkSkip,
};

// dyn_clone::DynClone +
pub trait Task: std::marker::Send + std::marker::Sync {
    fn name(&self) -> String;
    fn hooks(&self) -> &[super::Stage];
    fn check(&self, _: &mut AddonContext) -> Result<OkSkip, HEMTTError> {
        Ok((true, false))
    }
    fn check_single(&self, _: &mut AddonListContext) -> Result<(), HEMTTError> {
        Ok(())
    }
    fn prebuild(&self, _: &mut AddonContext) -> Result<OkSkip, HEMTTError> {
        Ok((true, false))
    }
    fn prebuild_single(&self, _: &mut AddonListContext) -> Result<(), HEMTTError> {
        Ok(())
    }
    fn build(&self, _: &mut AddonContext) -> Result<OkSkip, HEMTTError> {
        Ok((true, false))
    }
    fn build_single(&self, _: &mut AddonListContext) -> Result<(), HEMTTError> {
        Ok(())
    }
    fn postbuild(&self, _: &mut AddonContext) -> Result<OkSkip, HEMTTError> {
        Ok((true, false))
    }
    fn postbuild_single(&self, _: &mut AddonListContext) -> Result<(), HEMTTError> {
        Ok(())
    }
    fn release(&self, _: &mut AddonContext) -> Result<OkSkip, HEMTTError> {
        Ok((true, false))
    }
    fn release_single(&self, _: &mut AddonListContext) -> Result<(), HEMTTError> {
        Ok(())
    }
    fn postrelease(&self, _: &mut AddonContext) -> Result<OkSkip, HEMTTError> {
        Ok((true, false))
    }
    fn postrelease_single(&self, _: &mut AddonListContext) -> Result<(), HEMTTError> {
        Ok(())
    }
}
// dyn_clone::clone_trait_object!(Task);
