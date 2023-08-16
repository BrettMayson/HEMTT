use ariadne::{ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use hemtt_common::error::{tokens::Token, Code};
use lsp_types::{Diagnostic, Range};
use tracing::error;
use vfs::VfsPath;

#[allow(unused)]
/// Tried to create a [`FunctionDefinition`](crate::context::FunctionDefinition) that has multi-token arguments
///
/// ```cpp
/// #define FUNC(my arg) ...
/// ```
pub struct DefineMultiTokenArgument {
    /// The [`Token`] that was found
    pub(crate) token: Box<Token>,
    /// The [`Token`] stack trace
    pub(crate) trace: Vec<Token>,
    /// The arguments of the function
    pub(crate) arguments: Vec<Vec<Token>>,
}

impl Code for DefineMultiTokenArgument {
    fn ident(&self) -> &'static str {
        "PE5"
    }

    fn message(&self) -> String {
        "define with multi-token argument".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "define with multi-token argument `{}`",
            self.token.symbol().output().replace('\n', "\\n")
        )
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn generate_report(&self) -> Option<String> {
        let mut colors = ColorGenerator::default();
        let mut out = Vec::new();
        let span = self.token.source().start().0..self.token.source().end().0;
        let mut report = Report::build(
            ReportKind::Error,
            self.token.source().path_or_builtin(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message());
        for argument in &self.arguments {
            if argument.len() == 1 {
                continue;
            }
            let color = colors.next();
            report = report.with_label(
                Label::new((
                    self.token.source().path_or_builtin(),
                    argument[0].source().start().0..argument[argument.len() - 1].source().end().0,
                ))
                .with_color(color)
                .with_message(format!(
                    "try `{}`",
                    argument
                        .iter()
                        .filter(|t| !t.symbol().is_whitespace())
                        .map(|t| t.symbol().output())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .fg(color)
                )),
            );
        }
        if let Err(e) = report.finish().write_for_stdout(
            (
                self.token.source().path_or_builtin(),
                Source::from(self.token.source().path().map_or_else(String::new, |path| {
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
        let Some(path) = self.token.source().path() else {
            return None;
        };
        Some((
            path.clone(),
            self.diagnostic(Range {
                start: self.token.source().start().to_lsp(),
                end: self.token.source().end().to_lsp(),
            }),
        ))
    }
}
