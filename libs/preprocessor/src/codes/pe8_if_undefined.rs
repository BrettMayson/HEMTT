use std::sync::Arc;

// use ariadne::{ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use hemtt_common::reporting::{Code, Token};

use crate::{defines::Defines, Error};

#[allow(unused)]
/// Tried to use `#if` on an undefined macro
pub struct IfUndefined {
    /// The [`Token`] that was found
    token: Box<Token>,
    /// Similar defines
    similar: Vec<String>,
}

impl Code for IfUndefined {
    fn ident(&self) -> &'static str {
        "PE8"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "attempted to use `#if` on an undefined macro".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "undefined macro `{}`",
            self.token.symbol().to_string().replace('\n', "\\n")
        )
    }

    fn help(&self) -> Option<String> {
        if self.similar.is_empty() {
            None
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
}

impl IfUndefined {
    pub fn new(token: Box<Token>, defines: &Defines) -> Self {
        Self {
            similar: defines
                .similar_values(token.symbol().to_string().trim())
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
            token,
        }
    }

    pub fn code(token: Token, defines: &Defines) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token), defines)))
    }
}
