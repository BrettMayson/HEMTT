use hemtt_common::reporting::{Code, Diagnostic, Processed};

use crate::Class;

pub struct MissingParent {
    class: Class,
    diagnostic: Option<Diagnostic>,
}

// TODO: maybe we could have a `did you mean` here without too much trouble?

impl Code for MissingParent {
    fn ident(&self) -> &'static str {
        "CE7"
    }

    fn message(&self) -> String {
        "class's parent is not present".to_string()
    }

    fn label_message(&self) -> String {
        "not present in config".to_string()
    }

    fn help(&self) -> Option<String> {
        self.class.parent().map(|parent| {
            format!(
                "add `class {};` to the config to declare it as external",
                parent.as_str(),
            )
        })
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl MissingParent {
    pub fn new(class: Class, processed: &Processed) -> Self {
        Self {
            class,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let Some(parent) = self.class.parent() else {
            panic!("MissingParent::generate_processed called on class without parent");
        };
        self.diagnostic = Diagnostic::new_for_processed(&self, parent.span.clone(), processed);
        self
    }
}
