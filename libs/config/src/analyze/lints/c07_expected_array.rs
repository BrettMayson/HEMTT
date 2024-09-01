use std::sync::Arc;

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Diagnostic, Label, Processed},
};

use crate::{Property, Value};

#[allow(clippy::module_name_repetitions)]
pub struct LintC07ExpectedArray;

impl Lint for LintC07ExpectedArray {
    fn ident(&self) -> &str {
        "expected_array"
    }

    fn description(&self) -> &str {
        "Expected array"
    }

    fn documentation(&self) -> &str {
        "The property is expected to be an array"
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::error()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner for Runner {
    type Target = crate::Property;
    fn run(
        &self,
        _project: Option<&ProjectConfig>,
        _config: &LintConfig,
        processed: Option<&Processed>,
        target: &crate::Property,
    ) -> Vec<std::sync::Arc<dyn Code>> {
        let Some(processed) = processed else {
            return vec![];
        };
        let Property::Entry {
            value,
            expected_array,
            ..
        } = target
        else {
            return vec![];
        };
        if !expected_array {
            return vec![];
        }
        if let Value::Array(_) = value {
            return vec![];
        }
        // If we can't tell what the value is, we can't tell if it's an array or not
        if let Value::Invalid(_) = value {
            return vec![];
        }
        vec![Arc::new(Code07ExpectedArray::new(
            target.clone(),
            processed,
        ))]
    }
}

pub struct Code07ExpectedArray {
    property: Property,
    diagnostic: Option<Diagnostic>,
    suggestion: Option<String>,
}

impl Code for Code07ExpectedArray {
    fn ident(&self) -> &'static str {
        "L-C07"
    }

    fn message(&self) -> String {
        "property was expected to be an array".to_string()
    }

    fn label_message(&self) -> String {
        "expects an array".to_string()
    }

    fn help(&self) -> Option<String> {
        Some("remove the [] from the property".to_string())
    }

    fn suggestion(&self) -> Option<String> {
        self.suggestion.clone()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl Code07ExpectedArray {
    pub fn new(property: Property, processed: &Processed) -> Self {
        Self {
            property,
            diagnostic: None,
            suggestion: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let Property::Entry {
            name,
            value,
            expected_array,
        } = &self.property
        else {
            panic!("Code07ExpectedArray::generate_processed called on non-Code07ExpectedArray property");
        };
        assert!(
            expected_array,
            "Code07ExpectedArray::generate_processed called on non-Code07ExpectedArray property"
        );
        if let Value::Array(_) = value {
            panic!("Code07ExpectedArray::generate_processed called on non-Code07ExpectedArray property");
        }
        let ident_start = processed
            .mapping(name.span.start)
            .expect("mapping should exist");
        let ident_file = processed
            .source(ident_start.source())
            .expect("source should exist");
        let ident_end = processed
            .mapping(name.span.end)
            .expect("mapping should exist");
        let haystack = &processed.as_str()[ident_end.original_start()..value.span().start];
        let possible_end = ident_end.original_start() + haystack.find(']').unwrap_or(1) + 1;
        self.suggestion = Some(name.value.to_string());
        self.diagnostic = Diagnostic::new_for_processed(
            &self,
            ident_start.original_start()..possible_end,
            processed,
        );
        if let Some(diag) = &mut self.diagnostic {
            diag.labels.push(
                Label::secondary(ident_file.0.clone(), value.span()).with_message("not an array"),
            );
        }
        self
    }
}
