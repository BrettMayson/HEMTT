use hemtt_error::{tokens::Token, Code};

/// Expected an identifier, found something else
pub struct ExpectedIdent {
    /// The [`Token`] that was found
    pub(crate) token: Box<Token>,
    /// The [`Token`] stack trace
    pub(crate) trace: Vec<Token>,
}

impl Code for ExpectedIdent {
    fn ident(&self) -> &'static str {
        "PE3"
    }

    fn message(&self) -> String {
        "expected identifier".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "expected identifier, found `{}`",
            self.token.symbol().output().replace('\n', "\\n")
        )
    }

    fn help(&self) -> Option<String> {
        None
    }
}
