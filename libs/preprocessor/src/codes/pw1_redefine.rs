use hemtt_workspace::{reporting::{Code, Diagnostic, Label, Severity, Token}, WorkspacePath};

#[allow(unused)]
/// Unexpected token
pub struct RedefineMacro {
    /// The [`Token`] that was defined
    token: Box<Token>,
    token_file_source: Vec<WorkspacePath>,
    /// The original [`Token`] that was defined
    original: Box<Token>,
    original_file_source: Vec<WorkspacePath>,
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
        if self.token.position() == self.original.position() {
            return Some("A file is being included multiple times".to_string());
        }
        Some("`#undef` macros before redefining them".to_string())
    }

    fn note(&self) -> Option<String> {
        if self.token.position() != self.original.position() {
            return None;
        }
        let original = self.original_file_source.iter().map(WorkspacePath::as_str).collect::<Vec<_>>();
        let token = self.token_file_source.iter().map(WorkspacePath::as_str).collect::<Vec<_>>();
        let first_original = original.first().unwrap_or(&"");
        let last_original = original.last().unwrap_or(&"");
        let first_token = token.first().unwrap_or(&"");
        let last_token = token.last().unwrap_or(&"");
        let original = original.iter().skip(1).take(original.len() - 2).collect::<Vec<_>>();
        let token = token.iter().skip(1).take(token.len() - 2).collect::<Vec<_>>();
        let original = original.iter().map(|s| format!("\n ├── {s}")).collect::<Vec<_>>().join("\n");
        let token = token.iter().map(|s| format!("\n ├── {s}")).collect::<Vec<_>>().join("\n");
        Some(format!(
            "First from\n {first_original}{original}\n └── {last_original}\nSecond from\n {first_token}{token}\n └── {last_token}",
        ))
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
    #[must_use]
    pub const fn new(token: Box<Token>, token_file_source: Vec<WorkspacePath>, original: Box<Token>, original_file_source: Vec<WorkspacePath>) -> Self {
        Self { token, token_file_source, original, original_file_source }
    }
}
