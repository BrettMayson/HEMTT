use pest::iterators::Pair;

use super::{Rule, Token};

#[derive(Clone, Debug)]
pub struct TokenPos {
    start: (usize, (usize, usize)),
    end: (usize, (usize, usize)),
    path: String,
    token: Token,
}

impl TokenPos {
    pub fn new<S: Into<String>>(path: S, pair: Pair<'_, Rule>) -> Self {
        Self {
            start: (
                pair.as_span().start_pos().pos(),
                pair.as_span().start_pos().line_col(),
            ),
            end: (
                pair.as_span().end_pos().pos(),
                pair.as_span().end_pos().line_col(),
            ),
            path: path.into(),
            token: Token::from(pair),
        }
    }

    pub fn anon(token: Token) -> Self {
        Self {
            start: (0, (0, 0)),
            end: (0, (0, 0)),
            path: String::new(),
            token,
        }
    }

    pub fn with_pos(token: Token, pos: &Self) -> Self {
        Self {
            start: pos.start(),
            end: pos.end(),
            path: pos.path().to_string(),
            token,
        }
    }

    pub fn start(&self) -> (usize, (usize, usize)) {
        self.start
    }

    pub fn end(&self) -> (usize, (usize, usize)) {
        self.end
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn token(&self) -> &Token {
        &self.token
    }

    pub fn into_token(self) -> Token {
        self.token
    }
}

impl ToString for TokenPos {
    fn to_string(&self) -> String {
        self.token().to_string()
    }
}
