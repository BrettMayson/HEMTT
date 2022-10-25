use crate::whitespace::Whitespace;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Symbol {
    Word(String),
    Alpha(char),
    /// Parsed digits will always be a single digit, but generated digits may have multiple digits.
    Digit(usize),
    /// _
    Underscore,
    /// -
    Dash,
    /// =
    Assignment,
    /// +
    Plus,
    /// {
    LeftBrace,
    /// }
    RightBrace,
    /// [
    LeftBracket,
    /// ]
    RightBracket,
    /// (
    LeftParenthesis,
    /// )
    RightParenthesis,
    /// :
    Colon,
    /// ;
    Semicolon,
    /// ##
    Join,
    /// #
    Directive,
    /// \
    Escape,
    /// ,
    Comma,
    /// .
    Decimal,
    /// ""
    DoubleQuote,
    /// '
    SingleQuote,
    /// <
    LeftAngle,
    /// >
    RightAngle,

    Unicode(String),

    Newline,
    Whitespace(Whitespace),
    Comment(String),

    Eoi,
    Void,
}

impl Symbol {
    pub fn from_word<S: Into<String>>(word: S) -> Self {
        Self::Word(word.into())
    }

    #[must_use]
    pub const fn is_whitespace(&self) -> bool {
        matches!(&self, Self::Whitespace(_))
    }

    #[must_use]
    pub const fn is_newline(&self) -> bool {
        matches!(&self, Self::Newline)
    }

    #[must_use]
    pub fn output(&self) -> String {
        match *self {
            Self::Join => String::new(),
            _ => self.to_string(),
        }
    }
}

impl ToString for Symbol {
    fn to_string(&self) -> String {
        match self {
            Self::Word(w) => w.clone(),
            Self::Alpha(c) => c.to_string(),
            Self::Digit(d) => d.to_string(),
            Self::Underscore => "_".to_string(),
            Self::Dash => "-".to_string(),
            Self::Assignment => "=".to_string(),
            Self::Plus => "+".to_string(),
            Self::LeftBrace => "{".to_string(),
            Self::RightBrace => "}".to_string(),
            Self::LeftBracket => "[".to_string(),
            Self::RightBracket => "]".to_string(),
            Self::LeftParenthesis => "(".to_string(),
            Self::RightParenthesis => ")".to_string(),
            Self::Colon => ":".to_string(),
            Self::Semicolon => ";".to_string(),
            Self::Join => "##".to_string(),
            Self::Directive => "#".to_string(),
            Self::Escape => "\\".to_string(),
            Self::Comma => ",".to_string(),
            Self::Decimal => ".".to_string(),
            Self::DoubleQuote => "\"".to_string(),
            Self::SingleQuote => "'".to_string(),
            Self::LeftAngle => "<".to_string(),
            Self::RightAngle => ">".to_string(),
            Self::Unicode(s) => s.to_string(),
            Self::Newline => "\n".to_string(),
            Self::Whitespace(w) => w.to_string(),
            Self::Eoi | Self::Void | Self::Comment(_) => "".to_string(),
        }
    }
}
