use std::{ops::Range, sync::Arc};

use chumsky::span::Spanned;
use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner}, missing::check_is_missing_file, reporting::{Code, Diagnostic, Processed, Severity}
};

use crate::{analyze::LintData, Item, Value};

crate::analyze::lint!(LintC16FileMissing);

impl Lint<LintData> for LintC16FileMissing {
    fn ident(&self) -> &'static str {
        "file_missing"
    }
    fn sort(&self) -> u32 {
        160
    }
    fn description(&self) -> &'static str {
        "Checks for missing files referenced in config"
    }
    fn documentation(&self) -> &'static str {
        "### Explanation

Files should exists
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
    type Target = Spanned<crate::Value>;
    fn run(
        &self,
        project: Option<&ProjectConfig>,
        config: &LintConfig,
        processed: Option<&Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Spanned<crate::Value>,
        _data: &LintData,
    ) -> Vec<std::sync::Arc<dyn Code>> {
        let Some(project) = project else {
            return vec![];
        };
        let Some(processed) = processed else {
            return vec![];
        };
        let mut codes = Vec::new();

        match &target.inner {
            Value::Array(arr) => {
                for item in arr.items.iter() {
                    check_item(item, processed, project, config, &mut codes);
                }
            }
            Value::Str(str) => check_str(str, target.span.into_range(), processed, project, config, &mut codes),
            _ => {}
        }
        codes
    }
}

fn check_item(
    target: &Spanned<crate::Item>,
    processed: &Processed,
    project: &ProjectConfig,
    config: &LintConfig,
    codes: &mut Vec<Arc<dyn Code>>,
) {
    match &target.inner {
        Item::Array(items) => {
            for element in items {
                check_item(element, processed, project, config, codes);
            }
        }
        Item::Str(target_str) => {
            check_str(target_str, target.span.into_range(), processed, project, config, codes);
        }
        _ => {}
    }
}

fn check_str(
    target_str: &crate::Str,
    span: Range<usize>,
    processed: &Processed,
    project: &ProjectConfig,
    config: &LintConfig,
    codes: &mut Vec<Arc<dyn Code>>,
) {
    if !check_is_missing_file(target_str.value(), project, processed) {
        return;
    }
    let span = span.start + 1..span.end - 1;
    codes.push(Arc::new(Code16FileMissing::new(
        span,
        target_str.value().to_owned(),
        processed,
        config.severity(),
    )));
}

#[allow(clippy::module_name_repetitions)]
pub struct Code16FileMissing {
    span: Range<usize>,
    path: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for Code16FileMissing {
    fn ident(&self) -> &'static str {
        "L-C16"
    }
    fn link(&self) -> Option<&str> {
        Some("/lints/config.html#file_missing")
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn message(&self) -> String {
        "File Missing".to_string()
    }
    fn label_message(&self) -> String {
        "missing".to_string()
    }
    fn note(&self) -> Option<String> {
        Some(format!("file '{}' was not found in project", self.path))
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl Code16FileMissing {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        path: String,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            span,
            path,
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
