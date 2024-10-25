use std::{io::BufReader, sync::Arc};

use hemtt_stringtable::{
    analyze::{lint_addon, lint_addons, lint_check},
    Project,
};
use hemtt_workspace::reporting::{Code, Diagnostic};

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

        let stringtables = ctx
            .addons()
            .iter()
            .filter_map(|addon| {
                let path = ctx
                    .workspace_path()
                    .join(addon.folder())
                    .expect("vfs issue")
                    .join("stringtable.xml")
                    .expect("vfs issue");
                if path.exists().expect("vfs issue") {
                    let existing = path.read_to_string().expect("vfs issue");
                    match Project::from_reader(BufReader::new(existing.as_bytes())) {
                        Ok(project) => Some((project, addon.clone(), existing)),
                        Err(e) => {
                            debug!("Failed to parse stringtable for {}: {}", addon.folder(), e);
                            report.push(Arc::new(CodeStringtableInvalid::new(
                                addon.folder(),
                                e.to_string(),
                            )));
                            None
                        }
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        report.extend(lint_addons(&stringtables, Some(ctx.config())));

        for stringtable in stringtables {
            report.extend(lint_addon(&stringtable, Some(ctx.config())));
        }

        Ok(report)
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeStringtableInvalid {
    addon: String,
    reason: String,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeStringtableInvalid {
    fn ident(&self) -> &'static str {
        "INVALID-STRINGTABLE"
    }

    fn message(&self) -> String {
        format!("Stringtable in `{}` is invalid", self.addon)
    }

    fn note(&self) -> Option<String> {
        Some(self.reason.clone())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeStringtableInvalid {
    #[must_use]
    pub fn new(addon: String, reason: String) -> Self {
        Self {
            addon,
            reason,
            diagnostic: None,
        }
        .generate_processed()
    }

    fn generate_processed(mut self) -> Self {
        self.diagnostic = Some(Diagnostic::from_code(&self));
        self
    }
}
