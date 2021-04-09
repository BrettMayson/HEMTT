use regex::Regex;

use crate::{context::AddonContext, HEMTTError, Stage, Task};
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

    fn check(&self, ctx: &mut AddonContext) -> Result<(), HEMTTError> {
        let empty = std::fs::read_dir(ctx.addon().source())?.count() == 0;
        if empty {
            error!(
                "[{}] The addon source directory is empty: {}",
                ctx.addon().name(),
                ctx.addon().source()
            );
            unimplemented!()
        }
        Ok(())
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

    fn check(&self, ctx: &mut AddonContext) -> Result<(), HEMTTError> {
        // WARN: addon name standards
        let addon = ctx.addon();
        let p = ctx.global().project();
        let re = Regex::new(r"^([A-z0-9\-]+)$").unwrap();
        if !re.is_match(addon.name()) {
            ctx.warn("addon name is not following standards");
            ctx.info(&format!("try using `{}`", &addon.name().replace(" ", "_")));
        }
        // WARN: addons shouldn't start with the mod prefix
        if !p.prefix().is_empty() && addon.name().starts_with(p.prefix()) {
            ctx.warn("Redundant prefix in addon name");
            ctx.info(&format!(
                "use `{}`, pbos are prefixed automatically",
                if addon.name().starts_with(&format!("{}_", p.prefix())) {
                    &addon.name()[(p.prefix().len() + 1)..]
                } else {
                    &addon.name()[p.prefix().len()..]
                }
            ));
        }
        // WARN: compat outside of compat folder
        if addon.name().starts_with("compat") && addon.location() != AddonLocation::Compats {
            ctx.warn("[{}] compatibility addon should be in `compats/`");
        }
        Ok(())
    }
}
