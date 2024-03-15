use hemtt_workspace::reporting::{Code, Diagnostic, Label, Processed};

use crate::{Property, Value};

pub struct ExpectedArray {
    property: Property,
    diagnostic: Option<Diagnostic>,
    suggestion: Option<String>,
}

impl Code for ExpectedArray {
    fn ident(&self) -> &'static str {
        "CE6"
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

impl ExpectedArray {
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
            panic!("ExpectedArray::generate_processed called on non-ExpectedArray property");
        };
        assert!(
            expected_array,
            "ExpectedArray::generate_processed called on non-ExpectedArray property"
        );
        if let Value::Array(_) = value {
            panic!("ExpectedArray::generate_processed called on non-ExpectedArray property");
        }
        let ident_start = processed.mapping(name.span.start).unwrap();
        let ident_file = processed.source(ident_start.source()).unwrap();
        let ident_end = processed.mapping(name.span.end).unwrap();
        let haystack = &processed.as_str()[ident_end.original_start()..value.span().start];
        let possible_end =
            ident_end.original_start() + haystack.find(|c: char| c == ']').unwrap_or(1) + 1;
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
