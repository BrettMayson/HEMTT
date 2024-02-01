use std::ops::Range;

use hemtt_common::reporting::{Code, Diagnostic, Processed};

pub struct InvalidValueMacro {
    span: Range<usize>,
    diagnostic: Option<Diagnostic>,
}

impl Code for InvalidValueMacro {
    fn ident(&self) -> &'static str {
        "CE2"
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

impl InvalidValueMacro {
    pub fn new(span: Range<usize>, processed: &Processed) -> Self {
        Self {
            span,
            diagnostic: None,
        }
        .report_generate_processed(processed)
    }

    fn report_generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(&self, self.span.clone(), processed);
        // TODO add processed output to diagnostic
        self
        // let map = processed.mapping(self.span.start).unwrap();
        // let token = map.token();
        // let invalid = &processed.as_str()[self.span.start..self.span.end];
        // let mut out = Vec::new();
        // let mut colors = ColorGenerator::new();
        // let a = colors.next();
        // Report::build(
        //     ariadne::ReportKind::Error,
        //     token.position().path().as_str(),
        //     token.position().start().0,
        // )
        // .with_code(self.ident())
        // .with_message(self.message())
        // .with_label(
        //     Label::new((
        //         token.position().path().to_string(),
        //         token.position().start().0..token.position().end().0,
        //     ))
        //     .with_message(self.label_message())
        //     .with_color(a),
        // )
        // .with_help(self.help().unwrap())
        // .with_note(format!("The processed output was `{}`", invalid.fg(a)))
        // .finish()
        // .write_for_stdout(sources(processed.sources_adrianne()), &mut out)
        // .unwrap();
        // self.diagnostic = Some(String::from_utf8(out).unwrap());
        // self
    }
}
