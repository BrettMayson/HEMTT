use regex::Regex;

use crate::{Addon, AddonLocation, HEMTTError, OkSkip, Project, Stage, Task};

#[derive(Clone)]
pub struct NotEmpty {}
impl Task for NotEmpty {
    fn can_run(&self, _: &Addon, _: &Project, _: &Stage) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn parallel(&self, addon: &Addon, _: &Project, _: &Stage) -> Result<OkSkip, HEMTTError> {
        let mut ok = true;
        let empty = std::fs::read_dir(addon.folder())?.count() == 0;
        if empty {
            ok = false;
            error!("The addon directory `{}` is empty", addon.folder().display());
        }
        Ok((ok, false))
    }
}

#[derive(Clone)]
pub struct ValidName {}
impl Task for ValidName {
    fn can_run(&self, _: &Addon, _: &Project, _: &Stage) -> Result<bool, HEMTTError> {
        Ok(true)
    }

    fn parallel(&self, addon: &Addon, p: &Project, _: &Stage) -> Result<OkSkip, HEMTTError> {
        // WARN: addon name standards
        let re = Regex::new(r"^([A-z0-9\-]+)$").unwrap();
        if !re.is_match(&addon.name) {
            warn!("addon name `{}` is not following standards", &addon.name);
            info!("try using `{}`", &addon.name.replace(" ", "_"));
        }
        // WARN: addons shouldn't start with the mod prefix
        if !p.prefix.is_empty() && addon.name.starts_with(&p.prefix) {
            warn!("Redundant prefix in addon name `{}`", &addon.name);
            info!(
                "use `{}`, pbos are prefixed automatically",
                if addon.name.starts_with(&format!("{}_", &p.prefix)) {
                    &addon.name[(p.prefix.len() + 1)..]
                } else {
                    &addon.name[p.prefix.len()..]
                }
            );
        }
        // WARN: compat outside of compat folder
        if addon.name.starts_with("compat") && addon.location != AddonLocation::Compats {
            warn!("compatibility addon `{}` should be in `compats/`", &addon.name);
        }
        Ok((true, false))
    }
}
