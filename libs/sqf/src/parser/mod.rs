pub mod codes;
pub mod database;
pub mod lexer;

use std::ops::Range;
use std::sync::Arc;

use self::database::{is_special_command, Database};
use self::lexer::{Control, Operator, Token};
use crate::{BinaryCommand, Expression, NularCommand, Statement, Statements, UnaryCommand};

use chumsky::prelude::*;
use chumsky::Stream;
use hemtt_common::error::thiserror::Error;
use hemtt_common::reporting::{Code, Processed};

/// Parses a SQF string into a list of statements.
///
/// # Errors
/// [`ParserError::LexingError`] if the input string contains invalid tokens.
/// [`ParserError::ParsingError`] if the input string contains invalid syntax.
pub fn run(database: &Database, processed: &Processed) -> Result<Statements, ParserError> {
    let mut tokens = self::lexer::run(processed.as_str()).map_err(|e| {
        let mut errors: Vec<Arc<dyn Code>> = Vec::new();
        for e in e {
            errors.push(Arc::new(codes::spe1_invalid_token::InvalidToken::new(
                e.span(),
                processed,
            )));
        }
        ParserError::LexingError(errors)
    })?;
    self::lexer::strip_comments(&mut tokens);
    self::lexer::strip_noop(&mut tokens);
    let mut statements = run_for_tokens(database, processed, tokens).map_err(|e| {
        let mut errors: Vec<Arc<dyn Code>> = Vec::new();
        for e in e {
            errors.push(Arc::new(codes::spe2_unparseable::UnparseableSyntax::new(
                e.span(),
                processed,
            )));
        }
        ParserError::ParsingError(errors)
    })?;
    statements.source = processed.as_str().to_string();
    Ok(statements)
}

#[allow(clippy::range_plus_one)] // chumsky problem
/// Parses a list of tokens into a list of statements.
///
/// # Errors
/// [`ParserError::ParsingError`] if the input string contains invalid syntax.
pub fn run_for_tokens<I>(
    database: &Database,
    processed: &Processed,
    input: I,
) -> Result<Statements, Vec<Simple<Token>>>
where
    I: IntoIterator<Item = (Token, Range<usize>)>,
{
    let len = processed.as_str().len();
    let statements = parser(database).parse(Stream::from_iter(len..len + 1, input.into_iter()))?;
    Ok(statements)
}

fn parser(database: &Database) -> impl Parser<Token, Statements, Error = Simple<Token>> + '_ {
    statements(database).then_ignore(end())
}

#[allow(clippy::too_many_lines)]
fn statements(database: &Database) -> impl Parser<Token, Statements, Error = Simple<Token>> + '_ {
    recursive(|statements| {
        let expression = recursive(|expression| {
            let value = select! { |span|
                Token::Number(number) => Expression::Number(number, span),
                Token::String(string) => Expression::String(string, span),
              // i know you can *technically* redefine true and false to be something else in SQF,
              // so this isn't *technically* correct, but if you're doing evil things like that,
              // you don't deserve parity
              Token::Identifier(id) if id.eq_ignore_ascii_case("true") => Expression::Boolean(true, span),
              Token::Identifier(id) if id.eq_ignore_ascii_case("false") => Expression::Boolean(false, span),
              Token::Identifier(id) if database.has_nular_command(&id) => {
                Expression::NularCommand(NularCommand { name: id }, span)
              },
            };

            let array_open = just(Token::Control(Control::SquareBracketOpen));
            let array_close = just(Token::Control(Control::SquareBracketClose));
            let array = expression
                .clone()
                .separated_by(just(Token::Control(Control::Separator)))
                .allow_trailing()
                .map_with_span(Expression::Array)
                .delimited_by(array_open, array_close);

            let paren_open = just(Token::Control(Control::RoundBracketOpen));
            let paren_close = just(Token::Control(Control::RoundBracketClose));
            let parenthesized = expression.clone().delimited_by(paren_open, paren_close);

            let code_block_open = just(Token::Control(Control::CurlyBracketOpen));
            let code_block_close = just(Token::Control(Control::CurlyBracketClose));
            let code_block = statements
                .delimited_by(code_block_open, code_block_close)
                .map(Expression::Code);

            let variable = variable(database).map_with_span(Expression::Variable);
            let base = choice((value, array, parenthesized, code_block, variable));

            // Precedence 10 (Unary commands)
            let base = unary_command(database)
                .map_with_span(|value, span| (value, span))
                .repeated()
                .then(base)
                .foldr(|(unary_command, span), expression| {
                    Expression::UnaryCommand(unary_command, Box::new(expression), span)
                });

            let locate = |value: BinaryCommand, span: Range<usize>| (value, span);

            // Precedence 9 (Select)
            let base = apply_binary_command(base.boxed(), locate, {
                just(Token::Operator(Operator::Select)).to(BinaryCommand::Select)
            });

            // Precedence 8 (Exponent)
            let base = apply_binary_command(base.boxed(), locate, {
                just(Token::Operator(Operator::Exp)).to(BinaryCommand::Exp)
            });

            // Precedence 7 (Multiply, Divide, Remainder, Modulo, ATAN2)
            let base = apply_binary_command(
                base.boxed(),
                locate,
                select! {
                  Token::Operator(Operator::Mul) => BinaryCommand::Mul,
                  Token::Operator(Operator::Div) => BinaryCommand::Div,
                  Token::Operator(Operator::Rem) => BinaryCommand::Rem,
                  Token::Identifier(id) if id.eq_ignore_ascii_case("mod") => BinaryCommand::Mod,
                  Token::Identifier(id) if id.eq_ignore_ascii_case("atan2") => BinaryCommand::Atan2
                },
            );

            // Precedence 6 (Add, Subtract, Max, Min)
            let base = apply_binary_command(
                base.boxed(),
                locate,
                select! {
                  Token::Operator(Operator::Add) => BinaryCommand::Add,
                  Token::Operator(Operator::Sub) => BinaryCommand::Sub,
                  Token::Identifier(id) if id.eq_ignore_ascii_case("max") => BinaryCommand::Max,
                  Token::Identifier(id) if id.eq_ignore_ascii_case("min") => BinaryCommand::Min
                },
            );

            // Precedence 5 (Else)
            let base = apply_binary_command(base.boxed(), locate, {
                keyword("else").to(BinaryCommand::Else)
            });

            // Precedence 4 (All other binary operators)
            let base = apply_binary_command(base.boxed(), locate, binary_command(database));

            // Precedence 3 (Equals, Not Equals, Greater, Less, GreaterEquals, LessEquals, Config Path)
            let base = apply_binary_command(
                base.boxed(),
                locate,
                select! {
                  Token::Operator(Operator::Eq) => BinaryCommand::Eq,
                  Token::Operator(Operator::NotEq) => BinaryCommand::NotEq,
                  Token::Operator(Operator::Greater) => BinaryCommand::Greater,
                  Token::Operator(Operator::Less) => BinaryCommand::Less,
                  Token::Operator(Operator::GreaterEq) => BinaryCommand::GreaterEq,
                  Token::Operator(Operator::LessEq) => BinaryCommand::LessEq,
                  Token::Operator(Operator::ConfigPath) => BinaryCommand::ConfigPath
                },
            );

            // Precedence 2 (And)
            let base = apply_binary_command(
                base.boxed(),
                locate,
                select! {
                  Token::Operator(Operator::And) => BinaryCommand::And,
                  Token::Identifier(id) if id.eq_ignore_ascii_case("and") => BinaryCommand::And
                },
            );

            // Precedence 1 (Or)
            let base = apply_binary_command(
                base.boxed(),
                locate,
                select! {
                  Token::Operator(Operator::Or) => BinaryCommand::Or,
                  Token::Identifier(id) if id.eq_ignore_ascii_case("or") => BinaryCommand::Or
                },
            );

            base
        });

        // assignment without terminator, optionally including `private`
        let assignment = variable(database)
            .then_ignore(just(Token::Operator(Operator::Assign)))
            .then(expression.clone());
        let assignment = keyword("private")
            .or_not()
            .map(|v| v.is_some())
            .then(assignment)
            .map_with_span(|(local, (variable, expression)), span| {
                if local {
                    Statement::AssignLocal(variable, expression, span)
                } else {
                    Statement::AssignGlobal(variable, expression, span)
                }
            });
        assignment
            .or(expression.map_with_span(Statement::Expression))
            .separated_by(just(Token::Control(Control::Terminator)))
            .allow_trailing()
            .map(|content| Statements {
                content,
                source: String::new(),
            })
    })
}

fn apply_binary_command(
    base: impl Parser<Token, Expression, Error = Simple<Token>> + Clone,
    locate: impl Fn(BinaryCommand, Range<usize>) -> (BinaryCommand, Range<usize>),
    command: impl Parser<Token, BinaryCommand, Error = Simple<Token>>,
) -> impl Parser<Token, Expression, Error = Simple<Token>> {
    let command = command.map_with_span(locate);
    base.clone()
        .then(command.then(base).repeated())
        .foldl(|expr1, ((command, location), expr2)| {
            Expression::BinaryCommand(command, Box::new(expr1), Box::new(expr2), location)
        })
}

/// Matches unary commands, including special ones
fn unary_command(
    database: &Database,
) -> impl Parser<Token, UnaryCommand, Error = Simple<Token>> + '_ {
    select! {
      Token::Operator(Operator::Add) => UnaryCommand::Plus,
      Token::Operator(Operator::Sub) => UnaryCommand::Minus,
      Token::Operator(Operator::Not) => UnaryCommand::Not,
      Token::Identifier(id) if database.has_unary_command(&id) => UnaryCommand::Named(id)
    }
}

/// Matches binary commands, not including special ones
fn binary_command(
    database: &Database,
) -> impl Parser<Token, BinaryCommand, Error = Simple<Token>> + '_ {
    select! {
      Token::Operator(Operator::Associate) => BinaryCommand::Associate,
      Token::Identifier(id) if database.has_binary_command(&id) => BinaryCommand::Named(id)
    }
}

/// Matches any identifier that is a valid variable name (any that is not considered a command name)
fn variable(database: &Database) -> impl Parser<Token, String, Error = Simple<Token>> + '_ {
    select!(Token::Identifier(id) if !database.has_command(&id) && !is_special_command(&id) => id)
}

/// Matches a specific keyword identifier
fn keyword(name: &'static str) -> impl Parser<Token, (), Error = Simple<Token>> {
    select!(Token::Identifier(id) if id.eq_ignore_ascii_case(name) => ())
}

#[allow(clippy::module_name_repetitions)]
// TODO: `std::error::Error` implementation
#[derive(Debug, Error)]
pub enum ParserError {
    #[error("lexing error {0:?}")]
    LexingError(Vec<Arc<dyn Code>>),
    #[error("parsing error")]
    ParsingError(Vec<Arc<dyn Code>>),
}
