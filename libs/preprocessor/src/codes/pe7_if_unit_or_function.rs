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
    #[must_use]
    /// Create a new instance of `IfUnitOrFunction`
    /// 
    /// # Panics
    /// Panics if the token does not define anything in the defines
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

    #[must_use]
    pub fn code(token: Token, defines: &Defines) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token), defines)))
    }
}
