use hemtt_error::{
    tokens::{Symbol, Token},
    Code,
};

#[allow(unused)]
/// Unexpected token
pub struct UnexpectedToken {
    /// The [`Token`] that was found
    pub(crate) token: Box<Token>,
    /// The valid [`Symbol`]s that were expected
    pub(crate) expected: Vec<Symbol>,
    /// The [`Token`] stack trace
    pub(crate) trace: Vec<Token>,
}

impl Code for UnexpectedToken {
    fn ident(&self) -> &'static str {
        "PE1"
    }

    fn message(&self) -> String {
        "unexpected token".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "unexpected token `{}`",
            self.token.symbol().output().replace('\n', "\\n")
        )
    }

    fn help(&self) -> Option<String> {
        None
    }
}
