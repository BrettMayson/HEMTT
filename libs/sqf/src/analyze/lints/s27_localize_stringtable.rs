use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{analyze::LintData, Expression, UnaryCommand};

crate::analyze::lint!(LintS27LocalizeStringtable);

impl Lint<LintData> for LintS27LocalizeStringtable {
    fn ident(&self) -> &'static str {
        "localize_stringtable"
    }

    fn sort(&self) -> u32 {
        270
    }

    fn description(&self) -> &'static str {
        "trying to localize a stringtable that does not exist"
    }

    fn documentation(&self) -> &'static str {
        r"### Explanation
        Strings should exist...
        "
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::warning()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner<LintData> for Runner {
    type Target = crate::Expression;

    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        build_info: Option<&hemtt_common::config::BuildInfo>,
        config: &LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        target: &Self::Target,
        _data: &LintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        let Some(build_info) = build_info else {
            return Vec::new();
        };
        let Expression::UnaryCommand(UnaryCommand::Named(name), rhs, _) = target else {
            return Vec::new();
        };
        if name.to_lowercase() != "localize" {
            return Vec::new();
        }
        let Expression::String(lstring, range, _) = &**rhs else {
            return Vec::new();
        };
        if !build_info.stringtable_matches_project(lstring, false)
            || build_info.stringtable_exists(lstring, false)
        {
            return Vec::new();
        }
        vec![Arc::new(CodeS27LocalizeStringtable::new(
            lstring,
            range.clone(),
            processed,
            config.severity(),
        ))]
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS27LocalizeStringtable {
    raw: String,
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS27LocalizeStringtable {
    fn ident(&self) -> &'static str {
        "L-S27"
    }
    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#localize_stringtable")
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn message(&self) -> String {
        "invalid project stringtable entry for localize".to_string()
    }
    fn label_message(&self) -> String {
        String::new()
    }
    fn help(&self) -> Option<String> {
        Some(format!("[{}] not in project's stringtables", self.raw))
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS27LocalizeStringtable {
    #[must_use]
    pub fn new(raw: &str,span: Range<usize>, processed: &Processed, severity: Severity) -> Self {
        Self {
            raw: raw.into(),
            span,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
