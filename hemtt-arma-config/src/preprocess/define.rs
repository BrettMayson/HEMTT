use super::TokenPos;

#[derive(Debug, Clone)]
pub struct Define {
    pub call: bool,
    pub args: Option<Vec<Vec<TokenPos>>>,
    pub statement: Vec<TokenPos>,
}
