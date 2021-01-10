use regex::Regex;

use crate::{context::AddonContext, HEMTTError, OkSkip, Stage, Task};
use hemtt::AddonLocation;

#[derive(Clone)]
pub struct NotEmpty {}
impl Task for NotEmpty {
    fn name(&self) -> String {
        String::from("notempty")
    }

    fn hooks(&self) -> &[Stage] {
        &[Stage::Check]
    }

    fn check(&self, ctx: &mut AddonContext) -> Result<OkSkip, HEMTTError> {
        let mut ok = true;
        let empty = std::fs::read_dir(ctx.addon.source())?.count() == 0;
        if empty {
            ok = false;
            error!(
                "[{}] The addon source directory is empty: {}",
                ctx.addon.name(),
                ctx.addon.source()
            );
        }
        Ok((ok, false))
    }
}

#[derive(Clone)]
pub struct ValidName {}
impl Task for ValidName {
    fn name(&self) -> String {
        String::from("validname")
    }

    fn hooks(&self) -> &[Stage] {
        &[Stage::Check]
    }

    fn check(&self, ctx: &mut AddonContext) -> Result<OkSkip, HEMTTError> {
        // WARN: addon name standards
        let addon = ctx.addon;
        let p = ctx.global.project();
        let re = Regex::new(r"^([A-z0-9\-]+)$").unwrap();
        if !re.is_match(addon.name()) {
            warn!("[{}] addon name is not following standards", addon.name());
            info!("try using `{}`", &addon.name().replace(" ", "_"));
        }
        // WARN: addons shouldn't start with the mod prefix
        if !p.prefix().is_empty() && addon.name().starts_with(p.prefix()) {
            warn!("[{}] Redundant prefix in addon name", addon.name());
            info!(
                "use `{}`, pbos are prefixed automatically",
                if addon.name().starts_with(&format!("{}_", p.prefix())) {
                    &addon.name()[(p.prefix().len() + 1)..]
                } else {
                    &addon.name()[p.prefix().len()..]
                }
            );
        }
        // WARN: compat outside of compat folder
        if addon.name().starts_with("compat") && addon.location() != AddonLocation::Compats {
            warn!(
                "[{}] compatibility addon should be in `compats/`",
                addon.name()
            );
        }
        Ok((true, false))
    }
}
