use std::sync::Arc;

use crate::{
    span::Span,
    store::FileId,
    symbol::{CommentKind, Symbol},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub origin: TokenOrigin,
}

impl Token {
    #[must_use]
    /// Create a new token
    pub const fn new(kind: TokenKind, span: Span, origin: TokenOrigin) -> Self {
        Self { kind, span, origin }
    }

    #[must_use]
    /// Get the [`TokenKind`] of the token
    pub const fn kind(&self) -> &TokenKind {
        &self.kind
    }

    #[must_use]
    /// Get the [`Span`] of the token
    pub const fn span(&self) -> &Span {
        &self.span
    }

    #[must_use]
    /// Get the [`TokenOrigin`] of the token
    pub const fn origin(&self) -> &TokenOrigin {
        &self.origin
    }

    #[must_use]
    /// Extract the text content from this token regardless of its kind.
    pub fn text(&self) -> String {
        match &self.kind {
            TokenKind::Symbol(s) => s.to_string(),
            TokenKind::Comment(_, _) => String::new(),
            TokenKind::Pragma(pragma) => {
                let mut result = format!("#pragma {}", pragma.name);
                if !pragma.arguments.is_empty() {
                    result.push(' ');
                    result.push_str(&pragma.arguments.join(" "));
                }
                result
            }
        }
    }

    #[must_use]
    /// Get the [`Symbol`] of the token
    pub const fn as_symbol(&self) -> Option<&Symbol> {
        match &self.kind {
            TokenKind::Symbol(s) => Some(s),
            _ => None,
        }
    }

    #[cfg(any(test, feature = "test"))]
    /// Assert that the token is a [`Symbol`] and that it is equal to the expected symbol
    ///
    /// # Panics
    /// If the token is not a [`Symbol`] or if the symbol is not equal to the expected symbol, this function will panic with a message showing the expected and actual values.
    pub fn assert_eq_symbol(&self, expected: &Symbol) {
        assert_eq!(
            self.as_symbol(),
            Some(expected),
            "Expected symbol {:?}, but got {:?}",
            expected,
            self.kind
        );
    }

    #[cfg(any(test, feature = "test"))]
    /// Assert that the token is a [`Symbol`] and that it is equal to the expected symbol word
    ///
    /// # Panics
    /// If the token is not a [`Symbol`] or if the symbol is not a [`Word`](Symbol::Word), this function will panic with a message showing the expected and actual values.
    pub fn assert_eq_symbol_word(&self, expected: &str) {
        assert_eq!(
            self.as_symbol().map(|s| s.as_word()),
            Some(Some(expected)),
            "Expected symbol word {:?}, but got {:?}",
            expected,
            self.kind
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Symbol(Symbol),
    Comment(String, CommentKind),
    Pragma(PragmaKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PragmaKind {
    pub name: String,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenOrigin {
    Source(FileId),
    MacroExpansion {
        name: Arc<str>,
        call_site: Span,
        definition: Option<Span>,
    },
}

/// A type alias for the index of a token in a token stream.
pub type TokenIndex = usize;
