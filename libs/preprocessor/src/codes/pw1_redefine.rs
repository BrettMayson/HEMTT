use hemtt_workspace::reporting::{Code, Diagnostic, Label, Severity, Token};

#[allow(unused)]
/// Unexpected token
pub struct RedefineMacro {
    /// The [`Token`] that was defined
    token: Box<Token>,
    /// The original [`Token`] that was defined
    original: Box<Token>,
}

impl Code for RedefineMacro {
    fn ident(&self) -> &'static str {
        "PW1"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "redefining macro".to_string()
    }

    fn help(&self) -> Option<String> {
        Some("`#undef` macros before redefining them".to_string())
    }

    fn expand_diagnostic(&self, diag: Diagnostic) -> Diagnostic {
        diag.with_label(
            Label::secondary(
                self.original.position().path().clone(),
                self.original.position().span().start..self.original.position().span().end,
            )
            .with_message("previous definition here"),
        )
    }
}

impl RedefineMacro {
    pub const fn new(token: Box<Token>, original: Box<Token>) -> Self {
        Self { token, original }
    }
}
