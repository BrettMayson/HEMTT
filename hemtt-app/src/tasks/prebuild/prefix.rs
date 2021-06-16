use std::{collections::HashMap, sync::RwLock};

use crate::{
    context::{AddonContext, AddonListContext},
    HEMTTError, Stage, Task,
};

pub struct Prefix {
    seen: RwLock<HashMap<String, String>>,
}
impl Prefix {
    pub fn new() -> Self {
        Self {
            seen: RwLock::new(HashMap::new()),
        }
    }
}
impl Task for Prefix {
    fn name(&self) -> String {
        String::from("prefix")
    }

    fn hooks(&self) -> &[Stage] {
        &[Stage::Check, Stage::PreBuild]
    }

    fn check(&self, ctx: &mut AddonContext) -> Result<(), HEMTTError> {
        ctx.debug(&format!("prefix: {}", ctx.prefix()));
        if let Some(a) = self.seen.read().unwrap().get(&ctx.prefix().to_string()) {
            ctx.warn(&format!(
                "Prefix `{}` is already in use by `{}`!",
                ctx.prefix(),
                a
            ));
            return Ok(());
        }
        self.seen
            .write()
            .unwrap()
            .insert(ctx.prefix().to_string(), ctx.addon().source().to_string());
        Ok(())
    }

    fn prebuild_single(&self, ctx: &mut AddonListContext) -> Result<(), HEMTTError> {
        ctx.global()
            .container
            .set(PrefixMap(self.seen.read().unwrap().clone()));
        Ok(())
    }
}

pub struct PrefixMap(HashMap<String, String>);
impl PrefixMap {
    pub fn inner(&self) -> &HashMap<String, String> {
        &self.0
    }
}
