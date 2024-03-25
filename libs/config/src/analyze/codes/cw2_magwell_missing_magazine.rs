use std::ops::Range;

use hemtt_workspace::reporting::{Code, Diagnostic, Label, Processed};

use crate::Ident;

pub struct MagwellMissingMagazine {
    array: Ident,
    span: Range<usize>,

    diagnostic: Option<Diagnostic>,
}

// TODO: maybe we could have a `did you mean` here without too much trouble?

impl Code for MagwellMissingMagazine {
    fn ident(&self) -> &'static str {
        "CW2"
    }

    fn message(&self) -> String {
        "magazine defined in CfgMagazineWells was not found in CfgMagazines".to_string()
    }

    fn label_message(&self) -> String {
        "no matching magazine was found".to_string()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl MagwellMissingMagazine {
    pub fn new(array: Ident, span: Range<usize>, processed: &Processed) -> Self {
        Self {
            array,
            span,

            diagnostic: None,
        }
        .diagnostic_generate_processed(processed)
    }

    fn diagnostic_generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(&self, self.span.clone(), processed);
        if let Some(diag) = &mut self.diagnostic {
            diag.labels.push({
                let Some(map) = processed.mapping(self.array.span.start) else {
                    return self;
                };
                Label::secondary(map.original().path().clone(), map.original().span())
                    .with_message("magazine well defined here")
            });
        }
        self
    }
}
