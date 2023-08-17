// dead code from a previous hemtt version, don't feel the need to delete atm
#![allow(dead_code)]

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
    Equals,
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
}

impl Symbol {
    /// Create a new [`Word`](Symbol::Word) symbol
    pub fn from_word<S: Into<String>>(word: S) -> Self {
        Self::Word(word.into())
    }

    #[must_use]
    /// Check if a symbol is [`Word`](Symbol::Word)
    pub const fn is_word(&self) -> bool {
        matches!(self, Self::Word(_))
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
    /// Check if a symbol is [`Escape`](Symbol::Escape)
    pub const fn is_escape(&self) -> bool {
        matches!(&self, Self::Escape)
    }

    #[must_use]
    /// Check if a symbol is a [`Directive`](Symbol::Directive)
    pub const fn is_directive(&self) -> bool {
        matches!(self, Self::Directive)
    }

    #[must_use]
    /// Check if a symbol is [`LeftParenthesis`](Symbol::LeftParenthesis)
    pub const fn is_left_paren(&self) -> bool {
        matches!(self, Self::LeftParenthesis)
    }

    #[must_use]
    /// Check if a symbol is [`RightParenthesis`](Symbol::RightParenthesis)
    pub const fn is_right_paren(&self) -> bool {
        matches!(self, Self::RightParenthesis)
    }

    #[must_use]
    /// Check if a symbol is [`LeftAngle`](Symbol::LeftAngle)
    pub const fn is_left_angle(&self) -> bool {
        matches!(self, Self::LeftAngle)
    }

    #[must_use]
    /// Check if a symbol is [`RightAngle`](Symbol::RightAngle)
    pub const fn is_right_angle(&self) -> bool {
        matches!(self, Self::RightAngle)
    }

    #[must_use]
    /// Check if a symbol is [`Equals`](Symbol::Equals)
    pub const fn is_equals(&self) -> bool {
        matches!(self, Self::Equals)
    }

    #[must_use]
    /// Check if a symbol is [`Comma`](Symbol::Comma)
    pub const fn is_comma(&self) -> bool {
        matches!(self, Self::Comma)
    }

    #[must_use]
    /// Check if a symbol is an EOI
    pub const fn is_eoi(&self) -> bool {
        matches!(self, Self::Eoi)
    }

    #[must_use]
    /// Check if a symbol is [`Comment`](Symbol::Comment)
    pub const fn is_comment(&self) -> bool {
        matches!(self, Self::Comment(_))
    }

    #[must_use]
    /// Check if the symbol can be used to enclose #include paths
    pub const fn is_include_enclosure(&self) -> bool {
        matches!(self, Self::DoubleQuote | Self::LeftAngle)
    }

    #[must_use]
    /// Check if a symbol is [`DoubleQuote`](Symbol::DoubleQuote)
    pub const fn is_double_quote(&self) -> bool {
        matches!(self, Self::DoubleQuote)
    }

    #[must_use]
    /// Get the output of a symbol
    pub fn output(&self) -> String {
        match *self {
            Self::Join => String::new(),
            _ => self.to_string(),
        }
    }

    #[must_use]
    /// Get the opposite symbol of a symbol
    pub const fn matching_enclosure(&self) -> Option<Self> {
        match self {
            Self::LeftBrace => Some(Self::RightBrace),
            Self::RightBrace => Some(Self::LeftBrace),
            Self::LeftBracket => Some(Self::RightBracket),
            Self::RightBracket => Some(Self::LeftBracket),
            Self::LeftParenthesis => Some(Self::RightParenthesis),
            Self::RightParenthesis => Some(Self::LeftParenthesis),
            Self::LeftAngle => Some(Self::RightAngle),
            Self::RightAngle => Some(Self::LeftAngle),
            Self::DoubleQuote => Some(Self::DoubleQuote),
            _ => None,
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
            Self::Equals => "=".to_string(),
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
            Self::Eoi | Self::Comment(_) => String::new(),
        }
    }
}
