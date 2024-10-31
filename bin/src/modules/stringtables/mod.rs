use std::{io::BufReader, sync::Arc};

use hemtt_stringtable::{
    analyze::{lint_all, lint_check, lint_one},
    Project,
};
use hemtt_workspace::{
    reporting::{Code, Diagnostic},
    WorkspacePath,
};

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

        let mut stringtables = Vec::new();
        for root in ["addons", "optionals"] {
            let paths = ctx
                .workspace_path()
                .join(root)
                .expect("vfs issue")
                .walk_dir()
                .expect("vfs issue")
                .into_iter()
                .filter(|p| p.filename() == "stringtable.xml")
                .collect::<Vec<_>>();
            for path in paths {
                if path.exists().expect("vfs issue") {
                    let existing = path.read_to_string().expect("vfs issue");
                    match Project::from_reader(BufReader::new(existing.as_bytes())) {
                        Ok(project) => stringtables.push((project, path, existing)),
                        Err(e) => {
                            debug!("Failed to parse stringtable for {}: {}", path, e);
                            report.push(Arc::new(CodeStringtableInvalid::new(path, e.to_string())));
                        }
                    }
                }
            }
        }

        report.extend(lint_all(&stringtables, Some(ctx.config())));

        for stringtable in stringtables {
            report.extend(lint_one(&stringtable, Some(ctx.config())));
        }

        Ok(report)
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeStringtableInvalid {
    path: WorkspacePath,
    reason: String,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeStringtableInvalid {
    fn ident(&self) -> &'static str {
        "INVALID-STRINGTABLE"
    }

    fn message(&self) -> String {
        format!("Stringtable at `{}` is invalid", self.path)
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
    pub fn new(path: WorkspacePath, reason: String) -> Self {
        Self {
            path,
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
