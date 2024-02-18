use std::ops::Range;
use std::sync::Arc;

use chumsky::prelude::*;
use chumsky::text::ident;

pub type Tokens = Vec<(Token, Range<usize>)>;

macro_rules! chain_collect {
  ($Collect:ty: $($value:expr),+ $(,)?) => {
    std::iter::empty()$(.chain($value))+.collect::<$Collect>()
  };
}

pub fn strip_comments(tokens: &mut Tokens) {
    tokens.retain(|(token, _)| !matches!(token, Token::Comment(..)));
}

pub fn strip_noop(tokens: &mut Tokens) {
    let mut last_flag_delete = true;
    tokens.retain(|(token, _)| {
        let delete = matches!(token, Token::Control(Control::Terminator)) && last_flag_delete;
        last_flag_delete = matches!(
            token,
            Token::Control(Control::Terminator | Control::CurlyBracketOpen)
        );
        !delete
    });
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    /// Single-line or multi-line comment.
    /// Contains the entire contents of the comment.
    Comment(String),
    /// A pre-processor macro.
    Macro(String, String),
    /// An operator.
    Operator(Operator),
    /// A special control character.
    Control(Control),
    /// A scalar number literal.
    Number(crate::Scalar<f32>),
    /// An identifier, keyword, or command.
    Identifier(String),
    /// A string literal.
    String(Arc<str>),
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Control {
    Terminator,
    Separator,
    SquareBracketOpen,
    SquareBracketClose,
    RoundBracketOpen,
    RoundBracketClose,
    CurlyBracketOpen,
    CurlyBracketClose,
}

impl Control {
    #[must_use]
    pub const fn to_str(self) -> &'static str {
        match self {
            Self::Terminator => ";",
            Self::Separator => ",",
            Self::SquareBracketOpen => "[",
            Self::SquareBracketClose => "]",
            Self::RoundBracketOpen => "(",
            Self::RoundBracketClose => ")",
            Self::CurlyBracketOpen => "{",
            Self::CurlyBracketClose => "}",
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
    Not,
    Or,
    And,
    Eq,
    NotEq,
    Greater,
    Less,
    GreaterEq,
    LessEq,
    ConfigPath,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Exp,
    /// Equals `=`
    Assign,
    /// Colon `:`
    Associate,
    /// Pound sign `#`
    Select,
}

impl Operator {
    #[must_use]
    pub const fn to_str(self) -> &'static str {
        match self {
            Self::And => "&&",
            Self::Or => "||",
            Self::Eq => "==",
            Self::NotEq => "!=",
            Self::ConfigPath => ">>",
            Self::GreaterEq => ">=",
            Self::LessEq => "<=",
            Self::Greater => ">",
            Self::Less => "<",
            Self::Not => "!",
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Rem => "%",
            Self::Exp => "^",
            Self::Assign => "=",
            Self::Associate => ":",
            Self::Select => "#",
        }
    }
}

/// Lexes a raw SQF file into a token list, to be passed onto the parser stage.
///
/// # Errors
/// Returns chumsky's [`Simple`] error type, which is a wrapper around [`Range<usize>`] and [`String`].
pub fn run(input: impl AsRef<str>) -> Result<Tokens, Vec<Simple<char>>> {
    lexer().parse(input.as_ref())
}

fn lexer() -> impl Parser<char, Tokens, Error = Simple<char>> {
    end().padded().map(|()| vec![]).or(comment()
        .or(base())
        .map_with_span(|token, span| (token, span))
        .padded()
        .repeated()
        .then_ignore(end()))
}

fn base() -> impl Parser<char, Token, Error = Simple<char>> {
    let number = number().map(Token::Number);
    let identifier = ident().map(Token::Identifier);
    let string = string('\"').or(string('\'')).map(|s| Token::String(s.into()));

    // a constant (ident, number or string) must not be immediately followed
    // by another constant (without whitespace), or something is wrong
    let constant = choice((number, identifier, string)).boxed();
    let not_constant = constant.clone().not().ignored().or(end()).rewind();

    choice((
        operator().map(Token::Operator),
        control().map(Token::Control),
        constant.then_ignore(not_constant),
    ))
}

/// Captures comments
fn comment() -> impl Parser<char, Token, Error = Simple<char>> {
    choice((
        multi_line_comment().map(Token::Comment),
        single_line_comment().map(Token::Comment),
    ))
}

fn multi_line_comment() -> impl Parser<char, String, Error = Simple<char>> {
    just("/*").then(just("*/").not().repeated()).then(just("*/"))
    .map(|((prefix, comment), suffix)| {
        chain_collect!(String: prefix.chars(), comment, suffix.chars())
    })
}

fn single_line_comment() -> impl Parser<char, String, Error = Simple<char>> {
    just("//")
        .then(newline().not().repeated())
        .map(|(prefix, comment)| chain_collect!(String: prefix.chars(), comment))
}

fn newline() -> impl Parser<char, (), Error = Simple<char>> {
    just("\r\n").or(just("\n")).ignored()
}

#[allow(clippy::cast_precision_loss)]
fn number() -> impl Parser<char, crate::Scalar<f32>, Error = Simple<char>> {
    choice((
        number_float_exponent(),
        number_float_basic(),
        number_hex().map(|value| value as f32),
        number_int().map(|value| value as f32),
    ))
    .map(crate::Scalar)
}

fn number_hex() -> impl Parser<char, u32, Error = Simple<char>> {
    let digits = one_of("0123456789abcdefABCDEF").repeated().at_least(1);
    just("$")
        .or(just("0x"))
        .ignore_then(digits)
        .collect::<String>()
        .map(|value| u32::from_str_radix(&value, 16))
        .try_map(error_map)
}

fn number_int() -> impl Parser<char, u32, Error = Simple<char>> {
    number_digits()
        .collect::<String>()
        //.then_ignore(just('.').not().rewind())
        .from_str::<u32>()
        .try_map(error_map)
}

fn number_float_exponent() -> impl Parser<char, f32, Error = Simple<char>> {
    number_digits()
        .chain(just('.'))
        .or_not()
        .chain::<char, _, _>(number_digits())
        .chain::<char, _, _>(one_of("eE"))
        .chain::<char, _, _>(one_of("-+").or_not())
        .chain::<char, _, _>(number_digits())
        .collect::<String>()
        .from_str::<f32>()
        .try_map(error_map)
}

fn number_float_basic() -> impl Parser<char, f32, Error = Simple<char>> {
    number_digits()
        .or_not()
        .chain::<char, _, _>(just('.'))
        .chain::<char, _, _>(number_digits())
        .collect::<String>()
        .from_str::<f32>()
        .try_map(error_map)
}

fn number_digits() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    one_of("0123456789").repeated().at_least(1)
}

fn string(delimiter: char) -> impl Parser<char, String, Error = Simple<char>> {
    let content = just(delimiter).not().or(just([delimiter; 2]).to(delimiter));
    just(delimiter)
        .ignore_then(content.repeated())
        .then_ignore(just(delimiter))
        .collect()
}

fn operator() -> impl Parser<char, Operator, Error = Simple<char>> + Copy {
    choice((
        just("&&").to(Operator::And),
        just("||").to(Operator::Or),
        just("==").to(Operator::Eq),
        just("!=").to(Operator::NotEq),
        just(">>").to(Operator::ConfigPath),
        just(">=").to(Operator::GreaterEq),
        just("<=").to(Operator::LessEq),
        just(">").to(Operator::Greater),
        just("<").to(Operator::Less),
        just("!").to(Operator::Not),
        just("+").to(Operator::Add),
        just("-").to(Operator::Sub),
        just("*").to(Operator::Mul),
        just("/").to(Operator::Div),
        just("%").to(Operator::Rem),
        just("^").to(Operator::Exp),
        just("=").to(Operator::Assign),
        just(":").to(Operator::Associate),
        just("#").to(Operator::Select),
    ))
}

fn control() -> impl Parser<char, Control, Error = Simple<char>> + Copy {
    choice((
        just(";").to(Control::Terminator),
        just(",").to(Control::Separator),
        just("[").to(Control::SquareBracketOpen),
        just("]").to(Control::SquareBracketClose),
        just("(").to(Control::RoundBracketOpen),
        just(")").to(Control::RoundBracketClose),
        just("{").to(Control::CurlyBracketOpen),
        just("}").to(Control::CurlyBracketClose),
    ))
}

#[inline]
fn error_map<T, E, I>(result: Result<T, E>, span: Range<usize>) -> Result<T, Simple<I>>
where
    E: ToString,
    I: std::hash::Hash + Eq,
{
    result.map_err(|err| Simple::custom(span, err))
}
