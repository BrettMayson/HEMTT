use std::sync::Arc;

use ariadne::{sources, ColorGenerator, Fmt, Label, Report, ReportKind};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Token};

use crate::{defines::Defines, Error};

/// Tried to use `#if` on a [`Unit`](crate::context::Definition::Unit) or [`FunctionDefinition`](crate::context::Definition::Function)
pub struct IfUnitOrFunction {
    /// The [`Token`] that was found
    token: Box<Token>,
    /// Similar defines
    similar: Vec<String>,
    /// defined
    defined: (Token, bool),
    /// The report
    report: Option<String>,
}

impl Code for IfUnitOrFunction {
    fn ident(&self) -> &'static str {
        "PE7"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "attempted to use `#if` on a unit or function macro".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "unit or function macro `{}`",
            self.token.symbol().to_string().replace('\n', "\\n")
        )
    }

    fn report(&self) -> Option<String> {
        self.report.clone()
    }

    fn ci(&self) -> Vec<Annotation> {
        vec![self.annotation(
            AnnotationLevel::Error,
            self.token.position().path().as_str().to_string(),
            self.token.position(),
        )]
    }

    #[cfg(feature = "lsp")]
    fn generate_lsp(&self) -> Option<(VfsPath, Diagnostic)> {
        let Some(path) = self.token.position().path() else {
            return None;
        };
        Some((
            path.clone(),
            self.diagnostic(Range {
                start: self.token.position().start().to_lsp(),
                end: self.token.position().end().to_lsp(),
            }),
        ))
    }
}

impl IfUnitOrFunction {
    pub fn new(token: Box<Token>, defines: &Defines) -> Self {
        Self {
            similar: defines
                .similar_values(token.symbol().to_string().trim())
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
            defined: {
                let (t, d) = defines
                    .get_readonly(token.symbol().to_string().trim())
                    .unwrap();
                (t.as_ref().clone(), d.is_unit())
            },
            token,
            report: None,
        }
        .report_generate()
    }

    pub fn code(token: Token, defines: &Defines) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token), defines)))
    }

    fn report_generate(mut self) -> Self {
        let mut colors = ColorGenerator::default();
        let a = colors.next();
        let mut out = Vec::new();
        let span = self.token.position().span();
        let mut report = Report::build(
            ReportKind::Error,
            self.token.position().path().as_str(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((
                self.token.position().path().to_string(),
                span.start..span.end,
            ))
            .with_color(a)
            .with_message(if self.defined.1 {
                "trying to use a unit macro in an `#if`"
            } else {
                "trying to use a function macro in an `#if`"
            }),
        )
        .with_label(
            Label::new((
                self.defined.0.position().path().to_string(),
                self.defined.0.position().start().0..self.defined.0.position().end().0,
            ))
            .with_color(a)
            .with_message(format!(
                "defined as a {} here",
                if self.defined.1 { "unit" } else { "function" }
            )),
        );
        if self.similar.is_empty() {
            report = report.with_help(format!(
                "did you mean `#ifdef {}`",
                self.token.symbol().to_string().fg(a)
            ));
        } else {
            report = report.with_help(format!(
                "did you mean `{}`",
                self.similar
                    .iter()
                    .map(|dym| format!("{}", dym.fg(a)))
                    .collect::<Vec<_>>()
                    .join("`, `")
            ));
        }
        if let Err(e) = report.finish().write_for_stdout(
            sources(vec![
                (
                    self.token.position().path().to_string(),
                    self.token
                        .position()
                        .path()
                        .read_to_string()
                        .unwrap_or_default(),
                ),
                (
                    self.defined.0.position().path().to_string(),
                    self.defined
                        .0
                        .position()
                        .path()
                        .read_to_string()
                        .unwrap_or_default(),
                ),
            ]),
            &mut out,
        ) {
            panic!("while reporting: {e}");
        }
        self.report = Some(String::from_utf8(out).unwrap_or_default());
        self
    }
}
