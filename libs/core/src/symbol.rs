#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// The symbol of a [`Token`](crate::Token)
pub enum Symbol {
    /// A word is a contiguous sequence of letters, digits, and underscores.
    /// A word will never start with a digit.
    Word(std::sync::Arc<str>),
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

    /// A [`WhitespaceKind`] character
    Whitespace(WhitespaceKind),

    // TODO remove and replace with TokenKind
    Comment(String),

    /// A newline character
    Newline,

    /// End of input
    Eoi,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A kind of comment
pub enum CommentKind {
    /// A single line comment, starting with `//` and ending with a newline
    Line,
    /// A multi line comment, starting with `/*` and ending with `*/`
    Block,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A kind of whitespace
pub enum WhitespaceKind {
    /// A space character
    Space,
    /// A tab character
    Tab,
}

impl std::fmt::Display for WhitespaceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Space => write!(f, " "),
            Self::Tab => write!(f, "\t"),
        }
    }
}

impl Symbol {
    /// Create a new [`Word`](Symbol::Word) symbol
    pub fn from_word<S: Into<std::sync::Arc<str>>>(word: S) -> Self {
        Self::Word(word.into())
    }

    #[must_use]
    /// Check if a symbol is [`Word`](Symbol::Word)
    pub const fn is_word(&self) -> bool {
        matches!(self, Self::Word(_))
    }

    #[must_use]
    /// Get the word of a symbol if it is [`Word`](Symbol::Word)
    pub fn as_word(&self) -> Option<&str> {
        match self {
            Self::Word(w) => Some(&**w),
            _ => None,
        }
    }

    #[must_use]
    /// Check if a symbol is [`Whitespace`](Symbol::Whitespace) that is not a newline (space or tab)
    pub const fn is_whitespace(&self) -> bool {
        matches!(
            &self,
            Self::Whitespace(WhitespaceKind::Space | WhitespaceKind::Tab)
        )
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
    /// Check if a symbol is a comment
    pub const fn is_comment(&self) -> bool {
        matches!(self, Self::Comment(_))
    }

    #[must_use]
    /// Check if a symbol is a newline
    pub const fn is_newline(&self) -> bool {
        matches!(self, Self::Newline)
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
    /// Check if a symbol is [`SingleQuote`](Symbol::SingleQuote)
    pub const fn is_single_quote(&self) -> bool {
        matches!(self, Self::SingleQuote)
    }

    #[must_use]
    /// Check if a symbol is [`Join`](Symbol::Join)
    pub const fn is_join(&self) -> bool {
        matches!(self, Self::Join)
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

    #[cfg(any(test, feature = "test"))]
    /// Assert that two symbols are equal, ignoring the difference between [`Word`](Symbol::Word) and [`Arc<str>`](std::sync::Arc<str>)
    ///
    /// # Panics
    /// If the symbols are not equal, this function will panic with a message showing the two symbols that were compared.
    pub fn assert_eq(&self, other: &Self) {
        if self.is_word() && other.is_word() {
            assert_eq!(
                self.as_word().expect("must be word"),
                other.as_word().expect("must be word")
            );
        } else {
            assert_eq!(self, other);
        }
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Self::Alpha(c) = self {
            return write!(f, "{c}");
        }
        if let Self::Digit(d) = self {
            return write!(f, "{d}");
        }
        if let Self::Whitespace(w) = self {
            return write!(f, "{w}");
        }
        write!(
            f,
            "{}",
            match self {
                Self::Word(w) => w,
                Self::Underscore => "_",
                Self::Dash => "-",
                Self::Equals => "=",
                Self::Plus => "+",
                Self::LeftBrace => "{",
                Self::RightBrace => "}",
                Self::LeftBracket => "[",
                Self::RightBracket => "]",
                Self::LeftParenthesis => "(",
                Self::RightParenthesis => ")",
                Self::Colon => ":",
                Self::Semicolon => ";",
                Self::Join => "##",
                Self::Directive => "#",
                Self::Escape => "\\",
                Self::Slash => "/",
                Self::Comma => ",",
                Self::Decimal => ".",
                Self::DoubleQuote => "\"",
                Self::SingleQuote => "'",
                Self::LeftAngle => "<",
                Self::RightAngle => ">",
                Self::Unicode(s) => s,
                Self::Newline => "\n",
                Self::Eoi | Self::Comment(_) => "",
                Self::Alpha(_) | Self::Digit(_) | Self::Whitespace(_) => unreachable!(),
            }
        )
    }
}
