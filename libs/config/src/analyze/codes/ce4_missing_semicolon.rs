use std::ops::Range;

use ariadne::{sources, ColorGenerator, Fmt, Label, Report};
use hemtt_common::reporting::{Code, Processed};

pub struct MissingSemicolon {
    span: Range<usize>,
}

impl MissingSemicolon {
    pub const fn new(span: Range<usize>) -> Self {
        Self { span }
    }
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
        Some("add a semicolon `;` to the end of the property".to_string())
    }

    fn generate_processed_report(&self, processed: &Processed) -> Option<String> {
        let haystack = &processed.as_string()[self.span.clone()];
        let possible_end = self.span.start
            + haystack
                .find(|c: char| c == '\n')
                .unwrap_or_else(|| haystack.rfind(|c: char| c != ' ' && c != '}').unwrap_or(0) + 1);
        let map = processed.mapping(possible_end).unwrap();
        let token = map.token();
        let mut out = Vec::new();
        let mut colors = ColorGenerator::new();
        let a = colors.next();
        Report::build(
            ariadne::ReportKind::Error,
            token.position().path().as_str(),
            token.position().start().0,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            #[allow(clippy::range_plus_one)] // not supported by ariadne
            Label::new((
                token.position().path().to_string(),
                token.position().start().0..token.position().end().0,
            ))
            .with_message(format!("missing {}", "semicolon".fg(a)))
            .with_color(a),
        )
        .with_help(format!(
            "add a semicolon `{}` to the end of the property",
            ";".fg(a)
        ))
        .finish()
        .write_for_stdout(sources(processed.sources_adrianne()), &mut out)
        .unwrap();
        Some(String::from_utf8(out).unwrap())
    }

    #[cfg(feature = "lsp")]
    fn generate_processed_lsp(&self, processed: &Processed) -> Vec<(vfs::VfsPath, Diagnostic)> {
        let map = processed.mapping(self.span.end - 1).unwrap();
        let token = map.token().walk_up();
        let Some(path) = token.position().path() else {
            return vec![];
        };
        vec![(
            path.clone(),
            self.diagnostic(lsp_types::Range::new(
                token.position().start().to_lsp(),
                token.position().end().to_lsp(),
            )),
        )]
    }
}
