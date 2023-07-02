use hemtt_error::{tokens::Token, Code};

/// Unexpected end of file
pub struct UnexpectedEOF {
    /// The token that was found
    pub(crate) token: Box<Token>,
}

impl Code for UnexpectedEOF {
    fn ident(&self) -> &'static str {
        "PE2"
    }

    fn message(&self) -> String {
        "unexpected end of file".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "Unexpected end of file `{}`",
            self.token.symbol().output().replace('\n', "\\n")
        )
    }

    fn help(&self) -> Option<String> {
        None
    }
}
