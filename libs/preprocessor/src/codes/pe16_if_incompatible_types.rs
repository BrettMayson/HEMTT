use ariadne::{sources, ColorGenerator, Label, Report, ReportKind};
use hemtt_error::{tokens::Token, Code};
use lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Range};
use tracing::error;
use vfs::VfsPath;

#[allow(unused)]
/// Unexpected token
pub struct IfIncompatibleType {
    /// Left side of the operator
    pub(crate) left: (Vec<Token>, bool),
    /// Operator
    pub(crate) operator: Vec<Token>,
    /// Right side of the operator
    pub(crate) right: (Vec<Token>, bool),
    /// The [`Token`] stack trace
    pub(crate) trace: Vec<Token>,
}

impl Code for IfIncompatibleType {
    fn ident(&self) -> &'static str {
        "PE15"
    }

    fn message(&self) -> String {
        "incompatible types for operator in #if".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "incompatible types for operator in #if `{}` & `{}`",
            self.left
                .0
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<String>(),
            self.right
                .0
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<String>()
        )
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn generate_report(&self) -> Option<String> {
        let mut colors = ColorGenerator::default();
        let mut out = Vec::new();
        let span = self.operator.first().unwrap().source().start().0
            ..self.operator.last().unwrap().source().end().0;
        if let Err(e) = Report::build(
            ReportKind::Error,
            self.left.0.first().unwrap().source().path_or_builtin(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_labels({
            let mut labels = vec![Label::new((
                self.operator.first().unwrap().source().path_or_builtin(),
                span.start..span.end,
            ))
            .with_color(colors.next())
            .with_message("operator only supports numbers")];
            if self.left.1 {
                labels.push(
                    Label::new((
                        self.left.0.first().unwrap().source().path_or_builtin(),
                        self.left.0.first().unwrap().source().start().0
                            ..self.left.0.last().unwrap().source().end().0,
                    ))
                    .with_color(colors.next())
                    .with_message("left side of operator"),
                );
            }
            if self.right.1 {
                labels.push(
                    Label::new((
                        self.right.0.first().unwrap().source().path_or_builtin(),
                        self.right.0.first().unwrap().source().start().0
                            ..self.right.0.last().unwrap().source().end().0,
                    ))
                    .with_color(colors.next())
                    .with_message("right side of operator"),
                );
            }
            labels.into_iter()
        })
        .finish()
        .write_for_stdout(
            sources(
                vec![
                    (
                        self.operator.first().unwrap().source().path_or_builtin(),
                        self.operator
                            .first()
                            .unwrap()
                            .source()
                            .path()
                            .map_or_else(String::new, |path| {
                                path.read_to_string().unwrap_or_default()
                            }),
                    ),
                    (
                        self.left.0.first().unwrap().source().path_or_builtin(),
                        self.left
                            .0
                            .first()
                            .unwrap()
                            .source()
                            .path()
                            .map_or_else(String::new, |path| {
                                path.read_to_string().unwrap_or_default()
                            }),
                    ),
                    (
                        self.right.0.first().unwrap().source().path_or_builtin(),
                        self.right
                            .0
                            .first()
                            .unwrap()
                            .source()
                            .path()
                            .map_or_else(String::new, |path| {
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
        let Some(path) = self.left.0.first().unwrap().source().path() else {
            return None;
        };
        Some((
            path.clone(),
            Diagnostic {
                range: Range {
                    start: self.left.0.first().unwrap().source().start().to_lsp(),
                    end: self.right.0.last().unwrap().source().end().to_lsp(),
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
