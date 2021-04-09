use crate::{context::AddonContext, HEMTTError, Stage, Task};

pub struct Prefix {
    seen: Vec<String>,
}
impl Prefix {
    pub fn new() -> Self {
        Self { seen: Vec::new() }
    }
}
impl Task for Prefix {
    fn name(&self) -> String {
        String::from("prefix")
    }

    fn hooks(&self) -> &[Stage] {
        &[Stage::Check]
    }

    fn check(&self, ctx: &mut AddonContext) -> Result<(), HEMTTError> {
        ctx.debug(&format!("prefix: {}", ctx.prefix()));
        if self.seen.contains(&ctx.prefix().to_string()) {
            ctx.warn("Prefix is already in use!")
        }
        Ok(())
    }
}
