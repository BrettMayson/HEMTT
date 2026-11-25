use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Token};

use crate::Error;

#[allow(unused)]
/// Built-in macro is not supported by HEMTT
pub struct BuiltInNotSupported {
    /// The [`Token`] of the built-in macro
    token: Box<Token>,
    /// Used in an `#if`, `#ifdef` or `#ifndef`
    in_directive: bool,
}

impl Code for BuiltInNotSupported {
    fn ident(&self) -> &'static str {
        "PE26"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        format!("built-in macro `{}` is not supported by HEMTT", self.token.symbol())
    }

    fn note(&self) -> Option<String> {
        Some("certain built-in macros can not be rapified at build time\nHEMTT does not support them to prevent unexpected behaviour".to_string())
    }

    fn help(&self) -> Option<String> {
        self.in_directive.then(|| {
            String::from("use `#pragma hemtt flag pe26_ignore_unsupported_builtin`\nto have HEMTT act as if the condition is false.\nThe file will not be rapified or compiled to SQFC, so the game evaluates the condition,\nand the false branch will still be validated.")
        })
    }
}

impl BuiltInNotSupported {
    #[must_use]
    pub const fn new(token: Box<Token>) -> Self {
        Self {
            token,
            in_directive: false,
        }
    }

    #[must_use]
    pub fn code(token: Token) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token))))
    }

    #[must_use]
    /// Used in an `#if`, `#ifdef` or `#ifndef`, where the pragma flag can be used
    pub fn code_in_directive(token: Token) -> Error {
        Error::Code(Arc::new(Self {
            token: Box::new(token),
            in_directive: true,
        }))
    }
}
