use hemtt_workspace::reporting::{Code, Severity, Token};

#[allow(unused)]
pub struct IncludeForwardSlash {
    /// The [`Token`] representing the slash in the include path
    token: Box<Token>,
}

impl Code for IncludeForwardSlash {
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
        "Include path contains a forward_slash".to_string()
    }
    fn label_message(&self) -> String {
        "wrong slash type".to_string()
    }
    fn help(&self) -> Option<String> {
        Some("Arma expects include paths to use backslashes".to_string())
    }
}

impl IncludeForwardSlash {
    #[must_use]
    pub const fn new(token: Box<Token>) -> Self {
        Self { token }
    }
}
