use crate::{
    context::{AddonContext, AddonListContext},
    HEMTTError,
};

// dyn_clone::DynClone +
pub trait Task: std::marker::Send + std::marker::Sync {
    fn name(&self) -> String;
    fn hooks(&self) -> &[super::Stage];
    fn check(&self, _: &mut AddonContext) -> Result<(), HEMTTError> {
        Ok(())
    }
    fn check_single(&self, _: &mut AddonListContext) -> Result<(), HEMTTError> {
        Ok(())
    }
    fn prebuild(&self, _: &mut AddonContext) -> Result<(), HEMTTError> {
        Ok(())
    }
    fn prebuild_single(&self, _: &mut AddonListContext) -> Result<(), HEMTTError> {
        Ok(())
    }
    fn build(&self, _: &mut AddonContext) -> Result<(), HEMTTError> {
        Ok(())
    }
    fn build_single(&self, _: &mut AddonListContext) -> Result<(), HEMTTError> {
        Ok(())
    }
    fn postbuild(&self, _: &mut AddonContext) -> Result<(), HEMTTError> {
        Ok(())
    }
    fn postbuild_single(&self, _: &mut AddonListContext) -> Result<(), HEMTTError> {
        Ok(())
    }
    fn release(&self, _: &mut AddonContext) -> Result<(), HEMTTError> {
        Ok(())
    }
    fn release_single(&self, _: &mut AddonListContext) -> Result<(), HEMTTError> {
        Ok(())
    }
    fn postrelease(&self, _: &mut AddonContext) -> Result<(), HEMTTError> {
        Ok(())
    }
    fn postrelease_single(&self, _: &mut AddonListContext) -> Result<(), HEMTTError> {
        Ok(())
    }
}
// dyn_clone::clone_trait_object!(Task);
