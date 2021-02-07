#[derive(Parser)]
#[grammar = "preprocess/token/token.pest"]
pub struct PreProcessParser;

mod keyword;
pub use keyword::Keyword;
mod token_pos;
pub use token_pos::TokenPos;
mod whitespace;
pub use whitespace::Whitespace;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Keyword(Keyword),
    Word(String),
    Alpha(char),
    Digit(u8),
    Underscore,
    Dash,
    Assignment,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    LeftParenthesis,
    RightParenthesis,
    Colon,
    Semicolon,
    Directive,
    Escape,
    Comma,
    Decimal,
    DoubleQuote,
    SingleQuote,
    Char(char),

    Newline,
    Whitespace(Whitespace),

    EOI,
}

impl Token {
    pub fn from_word<S: Into<String>>(word: S) -> Token {
        let word = word.into();
        match word.as_str() {
            "class" => Token::Keyword(Keyword::Class),
            "delete" => Token::Keyword(Keyword::Delete),
            "enum" => Token::Keyword(Keyword::Enum),
            _ => Token::Word(word),
        }
    }

    pub fn is_whitespace(&self) -> bool {
        if let Self::Whitespace(_) = &self {
            true
        } else {
            false
        }
    }
}

impl From<pest::iterators::Pair<'_, Rule>> for Token {
    fn from(pair: pest::iterators::Pair<Rule>) -> Token {
        match pair.as_rule() {
            Rule::word => Token::from_word(pair.as_str().to_string()),
            Rule::alpha => Token::Alpha(pair.as_str().chars().next().unwrap()),
            Rule::digit => Token::Digit(pair.as_str().parse::<u8>().unwrap()),
            Rule::underscore => Token::Underscore,
            Rule::dash => Token::Dash,
            Rule::assignment => Token::Assignment,
            Rule::left_brace => Token::LeftBrace,
            Rule::right_brace => Token::RightBrace,
            Rule::left_bracket => Token::LeftBracket,
            Rule::right_bracket => Token::RightBracket,
            Rule::left_parentheses => Token::LeftParenthesis,
            Rule::right_parentheses => Token::RightParenthesis,
            Rule::colon => Token::Colon,
            Rule::semicolon => Token::Semicolon,
            Rule::directive => Token::Directive,
            Rule::escape => Token::Escape,
            Rule::comma => Token::Comma,
            Rule::decimal => Token::Decimal,
            Rule::double_quote => Token::DoubleQuote,
            Rule::single_quote => Token::SingleQuote,
            Rule::char => Token::Char(pair.as_str().chars().next().unwrap()),

            Rule::newline => Token::Newline,
            Rule::space => Token::Whitespace(Whitespace::Space),
            Rule::tab => Token::Whitespace(Whitespace::Tab),
            Rule::WHITESPACE => Token::from(pair.into_inner().next().unwrap()),
            Rule::EOI => Token::EOI,

            Rule::file => panic!("Unexpected attempt to tokenize file"),
            Rule::COMMENT => panic!("Unexpected attempt to tokenize comment"),
            // _ => panic!("Unknown: {:?}", pair),
        }
    }
}

impl ToString for Token {
    fn to_string(&self) -> String {
        match self {
            Token::Keyword(k) => k.to_string(),
            Token::Word(w) => w.to_owned(),
            Token::Alpha(c) => c.to_string(),
            Token::Digit(d) => d.to_string(),
            Token::Underscore => "_".to_string(),
            Token::Dash => "-".to_string(),
            Token::Assignment => "=".to_string(),
            Token::LeftBrace => "{".to_string(),
            Token::RightBrace => "}".to_string(),
            Token::LeftBracket => "[".to_string(),
            Token::RightBracket => "]".to_string(),
            Token::LeftParenthesis => "(".to_string(),
            Token::RightParenthesis => ")".to_string(),
            Token::Colon => ":".to_string(),
            Token::Semicolon => ";".to_string(),
            Token::Directive => "#".to_string(),
            Token::Escape => "\\".to_string(),
            Token::Comma => ",".to_string(),
            Token::Decimal => ".".to_string(),
            Token::DoubleQuote => "\"".to_string(),
            Token::SingleQuote => "'".to_string(),
            Token::Char(c) => c.to_string(),
            Token::Newline => "\n".to_string(),
            Token::Whitespace(w) => w.to_string(),
            Token::EOI => "".to_string(),
        }
    }
}
