use std::ops::Range;

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{analyze::LintData, Expression};

crate::analyze::lint!(LintS27LocalizeStringtable);

impl Lint<LintData> for LintS27LocalizeStringtable {
    fn display(&self) -> bool {
        false
    }

    fn ident(&self) -> &'static str {
        "localize_stringtable"
    }

    fn sort(&self) -> u32 {
        0
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
        _config: &LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        target: &Self::Target,
        data: &LintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        let Expression::String(lstring, range, _) = target else {
            return Vec::new();
        };
        
        if lstring.to_lowercase().starts_with("str_") || lstring.to_lowercase().starts_with("$str_") {
            let mut locations = data.localizations.lock().expect("mutex safety");
            let pos = if let Some(mapping) = processed.mapping(range.start) {
                mapping.token().position().clone()
            } else {
                // No position found for token
                return vec![];
            };
            locations.push((lstring.trim_start_matches('$').to_lowercase(), pos));
        }

        vec![]
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
