use hemtt_common::reporting::{Code, Diagnostic, Label, Processed};

use crate::model::Value;
use crate::Property;

pub struct UnexpectedArray {
    property: Property,
    diagnostic: Option<Diagnostic>,
    suggestion: Option<String>,
}

impl Code for UnexpectedArray {
    fn ident(&self) -> &'static str {
        "CE5"
    }

    fn message(&self) -> String {
        "property was not expected to be an array".to_string()
    }

    fn label_message(&self) -> String {
        "expected [] here".to_string()
    }

    fn help(&self) -> Option<String> {
        Some("add [] to the property".to_string())
    }

    fn suggestion(&self) -> Option<String> {
        self.suggestion.clone()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl UnexpectedArray {
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
            value: Value::UnexpectedArray(array),
            ..
        } = &self.property
        else {
            panic!("UnexpectedArray::generate_processed called on non-UnexpectedArray property");
        };
        let array_start = processed.mapping(array.span.start).unwrap();
        let array_file = processed.source(array_start.source()).unwrap();
        let ident_start = processed.mapping(name.span.start).unwrap();
        let ident_end = processed.mapping(name.span.end).unwrap();
        self.suggestion = Some(format!("{}[]", name.value));
        self.diagnostic = Diagnostic::new_for_processed(
            &self,
            ident_start.original_column()..ident_end.original_column(),
            processed,
        );
        if let Some(diag) = &mut self.diagnostic {
            diag.labels.push(
                Label::secondary(array_file.0.clone(), array.span.clone())
                    .with_message("unexpected array"),
            );
        }
        self
    }
}
