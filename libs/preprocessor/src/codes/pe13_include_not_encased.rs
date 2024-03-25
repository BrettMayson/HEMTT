use std::{rc::Rc, sync::Arc};

use hemtt_workspace::reporting::{Code, Diagnostic, Label, Token};

use crate::Error;

#[allow(unused)]
/// Unexpected token
pub struct IncludeNotEncased {
    /// The [`Token`] that was found
    token: Box<Token>,
    /// The [`Token`]s that make up the include
    path: Vec<Token>,
    /// The [`Symbol`] that the include is encased in
    start: Option<Token>,
}

impl Code for IncludeNotEncased {
    fn ident(&self) -> &'static str {
        "PE13"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "include not encased".to_string()
    }

    fn label_message(&self) -> String {
        "not encased".to_string()
    }

    fn suggestion(&self) -> Option<String> {
        self.start.as_ref()?;
        if self.path.is_empty() {
            return None;
        }
        Some(format!(
            "{}{}{}",
            self.start
                .as_ref()
                .map_or("<".to_string(), |t| t.symbol().to_string()),
            self.path
                .iter()
                .map(|t| t.symbol().to_string())
                .collect::<String>(),
            self.start.as_ref().map_or(">".to_string(), |t| t
                .symbol()
                .matching_enclosure()
                .expect("matching enclosure should exist if first exists")
                .to_string())
        ))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        let start = self.start.as_ref()?;
        let end = self.token.as_ref();
        let mut diag = Diagnostic::new(self.ident(), self.message()).with_label(
            Label::primary(
                start.position().path().clone(),
                start.position().span().start..end.position().span().end,
            )
            .with_message(self.label_message()),
        );
        if let Some(suggestion) = self.suggestion() {
            diag = diag.with_suggestion(suggestion);
        }
        Some(diag)
    }
}

impl IncludeNotEncased {
    pub fn new(token: Box<Token>, path: Vec<Rc<Token>>, start: Option<Token>) -> Self {
        Self {
            token,
            path: path.into_iter().map(|t| t.as_ref().clone()).collect(),
            start,
        }
    }

    pub fn code(token: Token, path: Vec<Rc<Token>>, start: Option<Token>) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token), path, start)))
    }
}
