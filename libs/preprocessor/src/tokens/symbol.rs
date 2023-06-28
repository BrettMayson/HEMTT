use crate::whitespace::Whitespace;

#[derive(Clone, Debug, PartialEq, Eq)]
/// The symbol of a [`Token`](crate::Token)
pub enum Symbol {
    /// A word is a contiguous sequence of letters, digits, and underscores.
    /// A word will never start with a digit.
    Word(String),
    /// A single alphanumeric character
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
    /// /
    Slash,
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

    /// A unicode character
    Unicode(String),

    /// A newline \n
    Newline,
    /// A [`Whitespace`] character
    Whitespace(Whitespace),
    /// A comment
    /// Comments are not parsed, but are kept in the token stream
    /// so that they can be outputted in the same format as the input.
    ///
    /// Comments have two forms:
    /// Single line comments start with `//` and end with a newline.
    /// Multi line comments start with `/*` and end with `*/`.
    Comment(String),

    /// End of input
    Eoi,
    /// Void token, not outputted
    Void,
}

impl Symbol {
    /// Create a new [`Word`](Symbol::Word) symbol
    pub fn from_word<S: Into<String>>(word: S) -> Self {
        Self::Word(word.into())
    }

    #[must_use]
    /// Check if a symbol is [`Whitespace`](Symbol::Whitespace) or [`Comment`](Symbol::Comment)
    pub const fn is_whitespace(&self) -> bool {
        matches!(&self, Self::Whitespace(_) | Self::Comment(_))
    }

    #[must_use]
    /// Check if a symbol is [`Newline`](Symbol::Newline)
    pub const fn is_newline(&self) -> bool {
        matches!(&self, Self::Newline)
    }

    #[must_use]
    /// Get the output of a symbol
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
            Self::Slash => "/".to_string(),
            Self::Comma => ",".to_string(),
            Self::Decimal => ".".to_string(),
            Self::DoubleQuote => "\"".to_string(),
            Self::SingleQuote => "'".to_string(),
            Self::LeftAngle => "<".to_string(),
            Self::RightAngle => ">".to_string(),
            Self::Unicode(s) => s.to_string(),
            Self::Newline => "\n".to_string(),
            Self::Whitespace(w) => w.to_string(),
            Self::Eoi | Self::Void | Self::Comment(_) => String::new(),
        }
    }
}
