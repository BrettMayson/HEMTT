use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use hemtt_error::{tokens::Token, Code};
use lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Range};
use tracing::error;
use vfs::VfsPath;

/// The [`Resolver`](crate::resolver::Resolver) could not find the target
pub struct IncludeNotFound {
    /// The target that was not found
    pub token: Vec<Token>,
    /// The [`Token`] stack trace
    pub trace: Vec<Token>,
}

impl Code for IncludeNotFound {
    fn ident(&self) -> &'static str {
        "PE12"
    }

    fn message(&self) -> String {
        "include not found".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "include not found `{}`",
            self.token
                .iter()
                .map(|t| t.symbol().output())
                .collect::<String>()
                .replace('\n', "\\n")
        )
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn generate_report(&self) -> Option<String> {
        let mut colors = ColorGenerator::default();
        let mut out = Vec::new();
        let span = self.token.first().unwrap().source().start().0
            ..self.token.last().unwrap().source().end().0;
        let token = self.token.first().unwrap();
        if let Err(e) = Report::build(
            ReportKind::Error,
            token.source().path_or_builtin(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((token.source().path_or_builtin(), span.start..span.end))
                .with_color(colors.next()),
        )
        .finish()
        .write_for_stdout(
            (
                token.source().path_or_builtin(),
                Source::from(token.source().path().map_or_else(String::new, |path| {
                    path.read_to_string().unwrap_or_default()
                })),
            ),
            &mut out,
        ) {
            error!("while reporting: {e}");
            return None;
        }
        Some(String::from_utf8(out).unwrap_or_default())
    }

    fn generate_lsp(&self) -> Option<(VfsPath, Diagnostic)> {
        let Some(path) = self.token.first().unwrap().source().path() else {
            return None;
        };
        Some((
            path.clone(),
            Diagnostic {
                range: Range::new(
                    self.token.first().unwrap().source().start().to_lsp(),
                    self.token.last().unwrap().source().end().to_lsp(),
                ),
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String(self.ident().to_string())),
                code_description: None,
                source: Some(String::from("HEMTT Preprocessor")),
                message: self.label_message(),
                related_information: None,
                tags: None,
                data: None,
            },
        ))
    }
}
