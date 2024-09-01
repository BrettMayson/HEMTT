use std::{ops::Range, sync::Arc};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Diagnostic, Processed},
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
    fn run(
        &self,
        _project: Option<&ProjectConfig>,
        _config: &LintConfig,
        processed: Option<&Processed>,
        target: &Value,
    ) -> Vec<Arc<dyn Code>> {
        let Some(processed) = processed else {
            return vec![];
        };
        if let Value::Invalid(invalid) = target {
            vec![if processed
                .mapping(invalid.start)
                .is_some_and(hemtt_workspace::reporting::Mapping::was_macro)
            {
                Arc::new(CodeC01InvalidValueMacro::new(invalid.clone(), processed))
            } else {
                Arc::new(CodeC01InvalidValue::new(invalid.clone(), processed))
            }]
        } else {
            vec![]
        }
    }
}

struct RunnerItem;
impl LintRunner for RunnerItem {
    type Target = Item;
    fn run(
        &self,
        _project: Option<&ProjectConfig>,
        _config: &LintConfig,
        processed: Option<&Processed>,
        target: &Item,
    ) -> Vec<Arc<dyn Code>> {
        let Some(processed) = processed else {
            return vec![];
        };
        if let Item::Invalid(invalid) = target {
            vec![if processed
                .mapping(invalid.start)
                .is_some_and(hemtt_workspace::reporting::Mapping::was_macro)
            {
                Arc::new(CodeC01InvalidValueMacro::new(invalid.clone(), processed))
            } else {
                Arc::new(CodeC01InvalidValue::new(invalid.clone(), processed))
            }]
        } else {
            vec![]
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeC01InvalidValue {
    span: Range<usize>,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeC01InvalidValue {
    fn ident(&self) -> &'static str {
        "L-C01"
    }

    fn message(&self) -> String {
        "property's value could not be parsed".to_string()
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
    pub fn new(span: Range<usize>, processed: &Processed) -> Self {
        Self {
            span,
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
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeC01InvalidValueMacro {
    fn ident(&self) -> &'static str {
        "L-C01M"
    }

    fn message(&self) -> String {
        "macro's result could not be parsed".to_string()
    }

    fn label_message(&self) -> String {
        "invalid macro result".to_string()
    }

    fn help(&self) -> Option<String> {
        Some("perhaps this macro has a `Q_` variant or you need `QUOTE(..)`".to_string())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeC01InvalidValueMacro {
    pub fn new(span: Range<usize>, processed: &Processed) -> Self {
        Self {
            span,
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
