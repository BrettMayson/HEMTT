use hemtt_common::reporting::{Code, Diagnostic, Severity, Token};

#[allow(unused)]
/// Unexpected token
pub struct PaddedArg {
    /// The [`Token`] that was found to be padding an arg
    token: Box<Token>,
    /// The identifier of macro that was being padded
    ident: String,
}

impl Code for PaddedArg {
    fn ident(&self) -> &'static str {
        "PW3"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "padding a macro argument".to_string()
    }

    fn expand_diagnostic(&self, diag: Diagnostic) -> Diagnostic {
        diag.with_note("padding a macro argument is likely unintended".to_string())
            .with_note(format!("occured in: `{}`", self.ident))
    }
}

impl PaddedArg {
    pub fn new(token: Box<Token>, ident: String) -> Self {
        Self { token, ident }
    }
}
