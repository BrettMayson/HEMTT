use hemtt_tokens::Token;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// An identifier
///
/// ```cpp
/// my_ident = 1;
/// ```
///
/// ```cpp
/// class my_ident {
///    ...
/// };
/// ```
pub struct Ident(Vec<Token>);

impl Ident {
    #[must_use]
    /// Create a new identifier
    pub fn new(tokens: Vec<Token>) -> Self {
        Self(tokens)
    }
}

impl ToString for Ident {
    fn to_string(&self) -> String {
        self.0
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<String>()
    }
}
