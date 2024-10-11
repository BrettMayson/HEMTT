use hemtt_workspace::reporting::{Code, Diagnostic, Label, Severity, Token};

#[allow(unused)]
/// Unexpected token
pub struct IncludeCase {
    /// The [`Token`] that included the file
    tokens: Vec<Token>,
    /// The name on disk of the file that was included
    ident: String,
}

impl Code for IncludeCase {
    fn ident(&self) -> &'static str {
        "PW4"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn token(&self) -> Option<&Token> {
        self.tokens.first()
    }

    fn message(&self) -> String {
        format!("on disk: `{}`", self.ident)
    }

    fn note(&self) -> Option<String> {
        Some("this will fail builds on platforms with case-sensitive filesystems".to_string())
    }

    fn expand_diagnostic(&self, diag: Diagnostic) -> Diagnostic {
        diag.clear_labels().with_label(
            Label::primary(
                self.tokens
                    .first()
                    .expect("at least one token")
                    .position()
                    .path()
                    .clone(),
                {
                    let first = self
                        .tokens
                        .first()
                        .expect("at least one token")
                        .position()
                        .span();
                    let last = self
                        .tokens
                        .last()
                        .expect("at least one token")
                        .position()
                        .span();
                    first.start..last.end
                },
            )
            .with_message(self.label_message()),
        )
    }
}

impl IncludeCase {
    #[must_use]
    pub const fn new(tokens: Vec<Token>, ident: String) -> Self {
        Self { tokens, ident }
    }
}
