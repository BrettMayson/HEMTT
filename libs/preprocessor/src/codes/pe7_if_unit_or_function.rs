use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic, Label, Token};

use crate::{defines::Defines, Error};

/// Tried to use `#if` on a [`Unit`](crate::context::Definition::Unit) or [`FunctionDefinition`](crate::context::Definition::Function)
pub struct IfUnitOrFunction {
    /// The [`Token`] that was found
    token: Box<Token>,
    /// Similar defines
    similar: Vec<String>,
    /// defined
    defined: (Token, bool),
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
            "attempted to use `#if` on {} macro `{}`",
            if self.defined.1 { "unit" } else { "function" },
            self.token.symbol().to_string().replace('\n', "\\n")
        )
    }

    fn help(&self) -> Option<String> {
        if self.similar.is_empty() {
            Some("did you mean to use `#ifdef`?".to_string())
        } else {
            Some(format!(
                "did you mean to use `{}`?",
                self.similar
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("`, `")
            ))
        }
    }

    fn suggestion(&self) -> Option<String> {
        if self.similar.is_empty() {
            Some(format!("#ifdef {}", self.token.symbol()))
        } else {
            None
        }
    }

    fn expand_diagnostic(&self, diag: Diagnostic) -> Diagnostic {
        diag.with_label(
            Label::secondary(
                self.defined.0.position().path().clone(),
                self.defined.0.position().span(),
            )
            .with_message(format!(
                "defined as a {} here",
                if self.defined.1 { "unit" } else { "function" }
            )),
        )
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
                let (t, d, _) = defines
                    .get_readonly(token.symbol().to_string().trim())
                    .expect("define should exist on error about its type");
                (t.as_ref().clone(), d.is_unit())
            },
            token,
        }
    }

    pub fn code(token: Token, defines: &Defines) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token), defines)))
    }

    // fn report_generate(mut self) -> Self {
    //     let mut colors = ColorGenerator::default();
    //     let a = colors.next();
    //     let mut out = Vec::new();
    //     let span = self.token.position().span();
    //     let mut report = Report::build(
    //         ReportKind::Error,
    //         self.token.position().path().as_str(),
    //         span.start,
    //     )
    //     .with_code(self.ident())
    //     .with_message(self.message())
    //     .with_label(
    //         Label::new((
    //             self.token.position().path().to_string(),
    //             span.start..span.end,
    //         ))
    //         .with_color(a)
    //         .with_message(if self.defined.1 {
    //             "trying to use a unit macro in an `#if`"
    //         } else {
    //             "trying to use a function macro in an `#if`"
    //         }),
    //     )
    //     .with_label(
    //         Label::new((
    //             self.defined.0.position().path().to_string(),
    //             self.defined.0.position().start().0..self.defined.0.position().end().0,
    //         ))
    //         .with_color(a)
    //         .with_message(format!(
    //             "defined as a {} here",
    //             if self.defined.1 { "unit" } else { "function" }
    //         )),
    //     );
    //     if self.similar.is_empty() {
    //         report = report.with_help(format!(
    //             "did you mean `#ifdef {}`",
    //             self.token.symbol().to_string().fg(a)
    //         ));
    //     } else {
    //         report = report.with_help(format!(
    //             "did you mean `{}`",
    //             self.similar
    //                 .iter()
    //                 .map(|dym| format!("{}", dym.fg(a)))
    //                 .collect::<Vec<_>>()
    //                 .join("`, `")
    //         ));
    //     }
    //     if let Err(e) = report.finish().write_for_stdout(
    //         sources(vec![
    //             (
    //                 self.token.position().path().to_string(),
    //                 self.token
    //                     .position()
    //                     .path()
    //                     .read_to_string()
    //                     .unwrap_or_default(),
    //             ),
    //             (
    //                 self.defined.0.position().path().to_string(),
    //                 self.defined
    //                     .0
    //                     .position()
    //                     .path()
    //                     .read_to_string()
    //                     .unwrap_or_default(),
    //             ),
    //         ]),
    //         &mut out,
    //     ) {
    //         panic!("while reporting: {e}");
    //     }
    //     self.report = Some(String::from_utf8(out).unwrap_or_default());
    //     self
    // }
}
