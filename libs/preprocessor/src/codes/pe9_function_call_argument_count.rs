use std::sync::Arc;

use hemtt_common::reporting::{Code, Diagnostic, Label, Token};

use crate::{defines::Defines, Error};

#[allow(unused)]
/// Tried to call a [`FunctionDefinition`](crate::context::FunctionDefinition) with the wrong number of arguments
pub struct FunctionCallArgumentCount {
    /// The [`Token`] that was found
    token: Box<Token>,
    /// The number of arguments that were expected
    expected: usize,
    /// The number of arguments that were found
    got: usize,
    /// Similar defines
    similar: Vec<String>,
    /// defined
    defined: (Token, Vec<Token>),
}

impl Code for FunctionCallArgumentCount {
    fn ident(&self) -> &'static str {
        "PE9"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "function call with incorrect number of arguments".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "called with {} argument{}",
            self.got,
            if self.got == 1 { "" } else { "s" }
        )
    }

    fn help(&self) -> Option<String> {
        if self.similar.is_empty() {
            None
        } else {
            Some(format!(
                "did you mean `{}`",
                self.similar
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("`, `")
            ))
        }
    }

    fn expand_diagnostic(&self, diag: Diagnostic) -> Diagnostic {
        diag.with_label(
            Label::secondary(
                self.token.position().path().clone(),
                self.defined.0.position().span(),
            )
            .with_message(format!(
                "defined with {} argument{}",
                self.defined.1.len(),
                if self.defined.1.len() == 1 { "" } else { "s" }
            )),
        )
    }
}

impl FunctionCallArgumentCount {
    pub fn new(token: Box<Token>, expected: usize, got: usize, defines: &Defines) -> Self {
        Self {
            expected,
            got,
            similar: defines
                .similar_values(token.symbol().to_string().trim())
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
            defined: {
                let (t, d) = defines
                    .get_readonly(token.symbol().to_string().trim())
                    .expect("define should exist on error about its type");
                (
                    t.as_ref().clone(),
                    d.as_function()
                        .expect("define should be a function in an error about it being called")
                        .clone()
                        .args()
                        .iter()
                        .map(|a| a.as_ref().clone())
                        .collect(),
                )
            },
            token,
        }
    }

    pub fn code(token: Token, expected: usize, got: usize, defines: &Defines) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token), expected, got, defines)))
    }
}
