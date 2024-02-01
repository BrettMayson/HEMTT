use std::ops::Range;

use hemtt_common::reporting::{diagnostic::Yellow, Code, Diagnostic, Processed};

pub struct MissingSemicolon {
    span: Range<usize>,
    diagnostic: Option<Diagnostic>,
}

impl Code for MissingSemicolon {
    fn ident(&self) -> &'static str {
        "CE4"
    }

    fn message(&self) -> String {
        "property is missing a semicolon".to_string()
    }

    fn label_message(&self) -> String {
        "missing semicolon".to_string()
    }

    fn help(&self) -> Option<String> {
        Some(format!(
            "add a semicolon {} to the end of the property",
            Yellow.paint(";")
        ))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl MissingSemicolon {
    pub fn new(span: Range<usize>, processed: &Processed) -> Self {
        Self {
            span,
            diagnostic: None,
        }
        .report_generate_processed(processed)
    }

    fn report_generate_processed(mut self, processed: &Processed) -> Self {
        let haystack = &processed.as_str()[self.span.clone()];
        let possible_end = self.span.start
            + haystack
                .find(|c: char| c == '\n')
                .unwrap_or_else(|| haystack.rfind(|c: char| c != ' ' && c != '}').unwrap_or(0) + 1);
        self.diagnostic =
            Diagnostic::new_for_processed(&self, possible_end..possible_end, processed);
        self
        //
        // let map = processed.mapping(possible_end).unwrap();
        // let token = map.token();
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
        //     #[allow(clippy::range_plus_one)] // not supported by ariadne
        //     Label::new((
        //         token.position().path().to_string(),
        //         token.position().start().0..token.position().end().0,
        //     ))
        //     .with_message(format!("missing {}", ";".fg(a)))
        //     .with_color(a),
        // )
        // .with_help(format!(
        //     "add a semicolon `{}` to the end of the property",
        //     ";".fg(a)
        // ))
        // .finish()
        // .write_for_stdout(sources(processed.sources_adrianne()), &mut out)
        // .unwrap();
        // self.diagnostic = Some(String::from_utf8(out).unwrap());
        // self
    }
}
