// Cleans existing files that are part of the hemtt project
#[derive(Clone)]
pub struct Clean {}
impl Task for Clean {
    fn can_run(&self, _: &Addon, _: &Project, _: &Stage) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn parallel(&self, addon: &Addon, p: &Project, _: &Stage) -> Result<OkSkip, HEMTTError> {
        let target = addon.target(p);
        if target.exists() {
            remove_file!(target)?;
        }
        Ok((true, false))
    }
}
