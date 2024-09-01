use std::{ops::Range, sync::Arc};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Diagnostic, Processed, Severity},
};

use crate::{Item, Value};

#[allow(clippy::module_name_repetitions)]
pub struct LintC01InvalidValue;

impl Lint for LintC01InvalidValue {
    fn ident(&self) -> &str {
        "invalid_value"
    }

    fn description(&self) -> &str {
        "Invalid value"
    }

    fn documentation(&self) -> &str {
        "The value is invalid"
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::error()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner>> {
        vec![Box::new(RunnerValue), Box::new(RunnerItem)]
    }
}

struct RunnerValue;

impl LintRunner for RunnerValue {
    type Target = Value;
    fn run_processed(
        &self,
        _project: Option<&ProjectConfig>,
        config: &LintConfig,
        processed: &Processed,
        target: &Value,
    ) -> Vec<Arc<dyn Code>> {
        if let Value::Invalid(invalid) = target {
            vec![if processed
                .mapping(invalid.start)
                .is_some_and(hemtt_workspace::reporting::Mapping::was_macro)
            {
                Arc::new(CodeC01InvalidValueMacro::new(
                    invalid.clone(),
                    processed,
                    config.severity(),
                ))
            } else {
                Arc::new(CodeC01InvalidValue::new(
                    invalid.clone(),
                    processed,
                    config.severity(),
                ))
            }]
        } else {
            vec![]
        }
    }
}

struct RunnerItem;
impl LintRunner for RunnerItem {
    type Target = Item;
    fn run_processed(
        &self,
        _project: Option<&ProjectConfig>,
        config: &LintConfig,
        processed: &Processed,
        target: &Item,
    ) -> Vec<Arc<dyn Code>> {
        if let Item::Invalid(invalid) = target {
            vec![if processed
                .mapping(invalid.start)
                .is_some_and(hemtt_workspace::reporting::Mapping::was_macro)
            {
                Arc::new(CodeC01InvalidValueMacro::new(
                    invalid.clone(),
                    processed,
                    config.severity(),
                ))
            } else {
                Arc::new(CodeC01InvalidValue::new(
                    invalid.clone(),
                    processed,
                    config.severity(),
                ))
            }]
        } else {
            vec![]
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeC01InvalidValue {
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeC01InvalidValue {
    fn ident(&self) -> &'static str {
        "L-C001"
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        "property's value could not be parsed.".to_string()
    }

    fn label_message(&self) -> String {
        "invalid value".to_string()
    }

    fn help(&self) -> Option<String> {
        Some("use quotes `\"` around the value".to_string())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeC01InvalidValue {
    pub fn new(span: Range<usize>, processed: &Processed, severity: Severity) -> Self {
        Self {
            span,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(&self, self.span.clone(), processed);
        self
    }
}

pub struct CodeC01InvalidValueMacro {
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeC01InvalidValueMacro {
    fn ident(&self) -> &'static str {
        "L-C001M"
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        "macro's value could not be parsed.".to_string()
    }

    fn label_message(&self) -> String {
        "invalid value".to_string()
    }

    fn help(&self) -> Option<String> {
        Some("use quotes `\"` around the value".to_string())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeC01InvalidValueMacro {
    pub fn new(span: Range<usize>, processed: &Processed, severity: Severity) -> Self {
        Self {
            span,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(&self, self.span.clone(), processed);
        if let Some(diag) = &mut self.diagnostic {
            diag.notes.push(format!(
                "The processed output was:\n{} ",
                &processed.as_str()[self.span.start..self.span.end]
            ));
        }
        self
    }
}
