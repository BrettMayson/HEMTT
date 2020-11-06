use super::Token;

#[derive(Debug, Clone)]
pub struct Define {
    pub call: bool,
    pub args: Option<Vec<Vec<Token>>>,
    pub statement: Vec<Token>,
}
