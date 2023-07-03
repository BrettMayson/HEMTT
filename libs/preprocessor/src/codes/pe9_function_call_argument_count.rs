use ariadne::{sources, ColorGenerator, Fmt, Label, Report, ReportKind};
use hemtt_error::{tokens::Token, Code};
use lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Range};
use tracing::error;
use vfs::VfsPath;

use crate::{Defines, DefinitionLibrary};

#[allow(unused)]
/// Tried to call a [`FunctionDefinition`](crate::context::FunctionDefinition) with the wrong number of arguments
pub struct FunctionCallArgumentCount {
    /// The [`Token`] that was found
    pub(crate) token: Box<Token>,
    /// The number of arguments that were expected
    pub(crate) expected: usize,
    /// The number of arguments that were found
    pub(crate) got: usize,
    /// The [`Token`] stack trace
    pub(crate) trace: Vec<Token>,
    /// The defines at the point of the error
    pub(crate) defines: Defines,
}

impl Code for FunctionCallArgumentCount {
    fn ident(&self) -> &'static str {
        "PE9"
    }

    fn message(&self) -> String {
        format!(
            "function call with incorrect number of arguments, expected `{}` got `{}`",
            self.expected, self.got
        )
    }

    fn label_message(&self) -> String {
        format!(
            "incorrect argument count, expected `{}` got `{}`",
            self.expected, self.got,
        )
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn generate_report(&self) -> Option<String> {
        let mut colors = ColorGenerator::default();
        let mut out = Vec::new();
        let span = self.token.source().start().0..self.token.source().end().0;
        let a = colors.next();
        let defined = self
            .defines
            .get(self.token.symbol().output().trim())
            .unwrap();
        let func = defined.1.as_function().unwrap();
        let did_you_mean = self
            .defines
            .similar_function(self.token.symbol().output().trim(), Some(self.got));
        let mut report = Report::build(
            ReportKind::Error,
            self.token.source().path_or_builtin(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((self.token.source().path_or_builtin(), span.start..span.end))
                .with_color(a)
                .with_message(format!(
                    "called with {} argument{} here",
                    self.got,
                    if self.got == 1 { "" } else { "s" }
                )),
        )
        .with_label(
            Label::new((
                defined.0.source().path_or_builtin(),
                defined.0.source().start().0..defined.0.source().end().0,
            ))
            .with_color(a)
            .with_message(format!(
                "defined here with {} argument{}",
                func.parameters().len(),
                if func.parameters().len() == 1 {
                    ""
                } else {
                    "s"
                }
            )),
        );
        if !did_you_mean.is_empty() {
            report = report.with_help(format!(
                "did you mean `{}`",
                did_you_mean
                    .into_iter()
                    .map(|dym| format!("{}", dym.fg(a)))
                    .collect::<Vec<_>>()
                    .join("`, `")
            ));
        }
        if let Err(e) = report.finish().write_for_stdout(
            sources(
                vec![
                    (
                        self.token.source().path_or_builtin(),
                        self.token.source().path().map_or_else(String::new, |path| {
                            path.read_to_string().unwrap_or_default()
                        }),
                    ),
                    (
                        defined.0.source().path_or_builtin(),
                        defined.0.source().path().map_or_else(String::new, |path| {
                            path.read_to_string().unwrap_or_default()
                        }),
                    ),
                ]
                .into_iter(),
            ),
            &mut out,
        ) {
            error!("while reporting: {e}");
            return None;
        }
        Some(String::from_utf8(out).unwrap_or_default())
    }

    fn generate_lsp(&self) -> Option<(VfsPath, Diagnostic)> {
        let Some(path) = self.token.source().path() else {
            return None;
        };
        Some((
            path.clone(),
            Diagnostic {
                range: Range {
                    start: self.token.source().start().to_lsp(),
                    end: self.token.source().end().to_lsp(),
                },
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
