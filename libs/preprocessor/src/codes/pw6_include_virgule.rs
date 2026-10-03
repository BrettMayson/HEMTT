use hemtt_workspace::reporting::{Code, Severity, Token};

#[allow(unused)]
pub struct IncludeVirgule {
    /// The [`Token`] representing the slash in the include path
    token: Box<Token>,
}

impl Code for IncludeVirgule {
    fn ident(&self) -> &'static str {
        "PW6"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }
    fn message(&self) -> String {
        "Include path contains a virgule slash".to_string()
    }
    fn label_message(&self) -> String {
        "slash in include path".to_string()
    }
}

impl IncludeVirgule {
    #[must_use]
    pub const fn new(token: Box<Token>) -> Self {
        Self { token }
    }
}
