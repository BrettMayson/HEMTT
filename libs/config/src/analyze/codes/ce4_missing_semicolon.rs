use std::ops::Range;

use ariadne::{sources, ColorGenerator, Label, Report};
use hemtt_error::{processed::Processed, Code};
use lsp_types::Diagnostic;

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
        let map = processed.original_col(self.span.end - 1).unwrap();
        let token = map.token().walk_up();
        let mut out = Vec::new();
        let mut colors = ColorGenerator::new();
        let a = colors.next();
        Report::build(
            ariadne::ReportKind::Error,
            token.source().path_or_builtin(),
            token.source().start().0,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            #[allow(clippy::range_plus_one)] // not supported by ariadne
            Label::new((
                token.source().path_or_builtin(),
                token.source().start().0..token.source().end().0 + 1,
            ))
            .with_message(self.label_message())
            .with_color(a),
        )
        .with_help(self.help().unwrap())
        .finish()
        .write_for_stdout(sources(processed.sources()), &mut out)
        .unwrap();
        Some(String::from_utf8(out).unwrap())
    }

    fn generate_processed_lsp(&self, processed: &Processed) -> Vec<(vfs::VfsPath, Diagnostic)> {
        let map = processed.original_col(self.span.end - 1).unwrap();
        let token = map.token().walk_up();
        let Some(path) = token.source().path() else {
            return vec![];
        };
        vec![(
            path.clone(),
            self.diagnostic(lsp_types::Range::new(
                token.source().start().to_lsp(),
                token.source().end().to_lsp(),
            ))
        )]
    }
}
