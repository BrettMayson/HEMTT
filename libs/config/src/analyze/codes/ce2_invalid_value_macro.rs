use std::ops::Range;

use ariadne::{sources, ColorGenerator, Fmt, Label, Report};
use hemtt_common::error::{processed::Processed, Code};
use lsp_types::Diagnostic;

pub struct InvalidValueMacro {
    span: Range<usize>,
}

impl InvalidValueMacro {
    pub const fn new(span: Range<usize>) -> Self {
        Self { span }
    }
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

    fn generate_processed_report(&self, processed: &Processed) -> Option<String> {
        let map = processed.original_col(self.span.start).unwrap();
        let token = map.token().walk_up();
        let invalid = &processed.output()[self.span.start..self.span.end];
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
            Label::new((
                token.source().path_or_builtin(),
                token.source().start().0..token.source().end().0,
            ))
            .with_message(self.label_message())
            .with_color(a),
        )
        .with_help(self.help().unwrap())
        .with_note(format!("The processed output was `{}`", invalid.fg(a)))
        .finish()
        .write_for_stdout(sources(processed.sources()), &mut out)
        .unwrap();
        Some(String::from_utf8(out).unwrap())
    }

    fn generate_processed_lsp(&self, processed: &Processed) -> Vec<(vfs::VfsPath, Diagnostic)> {
        let map = processed.original_col(self.span.start).unwrap();
        let token = map.token().walk_up();
        let Some(path) = token.source().path() else {
            return vec![];
        };
        vec![(
            path.clone(),
            self.diagnostic(lsp_types::Range::new(
                token.source().start().to_lsp(),
                token.source().end().to_lsp(),
            )),
        )]
    }
}
