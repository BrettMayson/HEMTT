use hemtt_workspace::reporting::{Code, Severity, Token};

#[allow(unused)]
pub struct UndefNotDefined {
    /// The [`Token`] that was never defined
    token: Box<Token>,
}

impl Code for UndefNotDefined {
    fn ident(&self) -> &'static str {
        "PW5"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }
    fn message(&self) -> String {
        "Undef not defined".to_string()
    }
    fn label_message(&self) -> String {
        "undefined macro".to_string()
    }
}

impl UndefNotDefined {
    #[must_use]
    pub const fn new(token: Box<Token>) -> Self {
        Self { token }
    }
}
