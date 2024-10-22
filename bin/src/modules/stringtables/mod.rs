use hemtt_stringtable::analyze::{lint_addons, lint_check};

use crate::{context::Context, report::Report, Error};

use super::Module;

#[derive(Debug, Default)]
pub struct Stringtables;
impl Stringtables {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Module for Stringtables {
    fn name(&self) -> &'static str {
        "Stringtables"
    }

    fn check(&self, ctx: &crate::context::Context) -> Result<crate::report::Report, crate::Error> {
        let mut report = Report::new();
        report.extend(lint_check(ctx.config().lints().sqf().clone()));
        Ok(report)
    }

    fn pre_build(&self, ctx: &Context) -> Result<Report, Error> {
        let mut report = Report::new();
        report.extend(lint_addons(
            ctx.workspace_path().to_owned(),
            &ctx.addons().to_vec(),
            Some(ctx.config()),
        ));
        Ok(report)
    }
}
