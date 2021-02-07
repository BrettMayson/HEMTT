use std::collections::HashMap;

use crate::preprocess::token::{Token, TokenPos};

pub type LineMap = Vec<(usize, usize, String, Token)>;

pub struct Rendered {
    tokens: Vec<TokenPos>,
    map: HashMap<usize, LineMap>,
}

impl Rendered {
    pub fn new(tokens: Vec<TokenPos>, map: HashMap<usize, LineMap>) -> Self {
        Self { tokens, map }
    }

    pub fn tokens(&self) -> &[TokenPos] {
        &self.tokens
    }

    pub fn map(&self) -> &HashMap<usize, LineMap> {
        &self.map
    }

    pub fn export(&self) -> String {
        let mut content = String::new();
        for token in &self.tokens {
            content.push_str(&token.to_string());
        }
        content
    }

    #[cfg(feature = "maps")]
    pub fn export_map_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.map)
    }

    #[cfg(feature = "maps")]
    pub fn export_html(&self) -> String {
        let mut content = String::new();
        for token in &self.tokens {
            let title = format!("Source: {} {:?}", token.path(), token.start().1);
            println!("D> {:?}", token.token());
            match token.token() {
                Token::Keyword(_) => content.push_str(&format!("<span class=\"keyword\" title=\"{}\">{}</span>", title, token.to_string())),
                _ => content.push_str(&format!("<span title=\"{}\">{}</span>", title, token.to_string())),
                // Token::Word(_) => {}
                // Token::Alpha(_) => {}
                // Token::Underscore => {}
                // Token::Dash => {}
                // Token::Assignment => {}
                // Token::LeftBrace => {}
                // Token::RightBrace => {}
                // Token::LeftBracket => {}
                // Token::RightBracket => {}
                // Token::LeftParenthesis => {}
                // Token::RightParenthesis => {}
                // Token::Colon => {}
                // Token::Semicolon => {}
                // Token::Directive => {}
                // Token::Escape => {}
                // Token::Comma => {}
                // Token::Decimal => {}
                // Token::DoubleQuote => {}
                // Token::SingleQuote => {}
                // Token::Char(_) => {}
                // Token::Newline => {}
                // Token::Whitespace(_) => {}
                // Token::EOI => {}
            }
        }
        super::html::wrap(content)
    }
}
