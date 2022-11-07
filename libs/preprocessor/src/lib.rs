#![deny(clippy::all, clippy::nursery)]
#![warn(clippy::pedantic)]

//! Arma 3 config preprocessor.

use std::{iter::Peekable, path::Path};

use context::{Context, Definition, FunctionDefinition};
use hemtt_tokens::whitespace;
use hemtt_tokens::{symbol::Symbol, Token};
use ifstate::IfState;

mod context;
mod error;
mod ifstate;
mod map;
mod parse;
mod resolver;

pub use error::Error;
pub use map::{Mapping, Processed};
pub use resolver::resolvers;
pub use resolver::{
    resolvers::{LocalResolver, NoResolver},
    Resolver,
};

/// Preprocesses a config file.
/// # Errors
/// it can fail
/// # Panics
/// it can panic
pub fn preprocess_file<R>(entry: &str, resolver: &mut R) -> Result<Vec<Token>, Error>
where
    R: Resolver,
{
    let source = resolver.find_include(
        Path::new(entry).parent().unwrap().to_str().unwrap(),
        entry,
        Path::new(entry).file_name().unwrap().to_str().unwrap(),
        vec![Token::builtin()],
    )?;
    let mut tokens = crate::parse::parse(entry, &source.1)?;
    let eoi = tokens.pop().unwrap();
    tokens.push(Token::ending_newline());
    tokens.push(eoi);
    let mut context = Context::new(entry.to_string());
    let mut tokenstream = tokens.into_iter().peekable();
    root_preprocess(resolver, &mut context, &mut tokenstream, false)
}

/// # Errors
/// it can fail
pub fn preprocess_string(source: &str) -> Result<Vec<Token>, Error> {
    let tokens = crate::parse::parse("%anonymous%", source)?;
    let mut context = Context::new(String::from("%anonymous%"));
    let mut tokenstream = tokens.into_iter().peekable();
    root_preprocess(
        &mut NoResolver::new(),
        &mut context,
        &mut tokenstream,
        false,
    )
}

fn root_preprocess<R>(
    resolver: &mut R,
    context: &mut Context,
    tokenstream: &mut Peekable<impl Iterator<Item = Token>>,
    allow_quote: bool,
) -> Result<Vec<Token>, Error>
where
    R: Resolver,
{
    let mut output = Vec::new();
    while let Some(token) = tokenstream.peek() {
        match token.symbol() {
            Symbol::Directive => {
                output.append(&mut directive_preprocess(
                    resolver,
                    context,
                    tokenstream,
                    allow_quote,
                )?);
            }
            Symbol::Comment(_) => {
                tokenstream.next();
            }
            _ => {
                if context.ifstates().reading() {
                    output.append(&mut walk_line(resolver, context, tokenstream)?);
                } else {
                    tokenstream.next();
                }
            }
        }
    }
    Ok(output)
}

fn directive_preprocess<R>(
    resolver: &mut R,
    context: &mut Context,
    tokenstream: &mut Peekable<impl Iterator<Item = Token>>,
    allow_quote: bool,
) -> Result<Vec<Token>, Error>
where
    R: Resolver,
{
    if let Some(token) = tokenstream.peek() {
        match token.symbol() {
            Symbol::Directive => {}
            _ => {
                return Err(Error::UnexpectedToken {
                    token: Box::new(token.clone()),
                    expected: vec![Symbol::Directive],
                })
            }
        }
    } else {
        return Err(Error::UnexpectedEOF);
    }
    let mut output = Vec::new();
    tokenstream.next();
    if let Some(token) = tokenstream.next() {
        if let Symbol::Word(command) = token.symbol() {
            match (command.as_str(), context.ifstates().reading()) {
                ("include", true) => {
                    whitespace::skip(tokenstream);
                    output.append(&mut directive_include_preprocess(
                        resolver,
                        context,
                        tokenstream,
                    )?);
                }
                ("define", true) => {
                    whitespace::skip(tokenstream);
                    directive_define_preprocess(resolver, context, tokenstream)?;
                }
                ("undef", true) => {
                    whitespace::skip(tokenstream);
                    directive_undef_preprocess(context, tokenstream)?;
                }
                ("if", true) => {
                    whitespace::skip(tokenstream);
                    directive_if_preprocess(context, tokenstream)?;
                }
                ("ifdef", true) => {
                    whitespace::skip(tokenstream);
                    directive_ifdef_preprocess(context, tokenstream, true)?;
                }
                ("ifndef", true) => {
                    whitespace::skip(tokenstream);
                    directive_ifdef_preprocess(context, tokenstream, false)?;
                }
                ("ifdef" | "ifndef", false) => {
                    context.ifstates_mut().push(IfState::PassingChild);
                    whitespace::skip(tokenstream);
                    tokenstream.next();
                    eat_newline(tokenstream)?;
                }
                ("else", _) => {
                    context.ifstates_mut().flip();
                    eat_newline(tokenstream)?;
                }
                ("endif", _) => {
                    context.ifstates_mut().pop();
                    eat_newline(tokenstream)?;
                }
                (_, true) => {
                    if allow_quote {
                        let source = token.source().clone();
                        output.push(Token::new(Symbol::DoubleQuote, source.clone()));
                        if let Symbol::Word(word) = token.symbol() {
                            if let Some((_source, definition)) = context.get(word, &token) {
                                output.append(&mut walk_definition(
                                    resolver,
                                    context,
                                    tokenstream,
                                    token,
                                    definition,
                                )?);
                            } else {
                                output.push(token);
                            }
                        } else {
                            output.push(token);
                        }
                        output.push(Token::new(Symbol::DoubleQuote, source));
                    } else {
                        return Err(Error::UnknownDirective {
                            directive: Box::new(token),
                        });
                    }
                }
                _ => {}
            }
        }
    } else {
        return Err(Error::UnexpectedEOF);
    }
    Ok(output)
}

fn directive_include_preprocess<R>(
    resolver: &mut R,
    context: &mut Context,
    tokenstream: &mut Peekable<impl Iterator<Item = Token>>,
) -> Result<Vec<Token>, Error>
where
    R: Resolver,
{
    let encased_in = match tokenstream.peek().unwrap().symbol() {
        Symbol::DoubleQuote | Symbol::SingleQuote => tokenstream.next().unwrap().symbol().clone(),
        Symbol::LeftAngle => {
            tokenstream.next();
            Symbol::RightAngle
        }
        _ => {
            return Err(Error::UnexpectedToken {
                token: Box::new(tokenstream.peek().unwrap().clone()),
                expected: vec![Symbol::DoubleQuote, Symbol::SingleQuote, Symbol::LeftAngle],
            })
        }
    };
    let mut path = String::new();
    let mut path_tokens = Vec::new();
    while let Some(token) = tokenstream.peek() {
        if token.symbol() == &encased_in {
            tokenstream.next();
            break;
        }
        if token.symbol() == &Symbol::Newline {
            return Err(Error::UnexpectedToken {
                token: Box::new(token.clone()),
                expected: vec![encased_in],
            });
        }
        path.push_str(token.to_string().as_str());
        path_tokens.push(token.clone());
        tokenstream.next();
    }
    if tokenstream.peek().is_none() {
        return Err(Error::UnexpectedEOF);
    }
    let (pathbuf, mut tokens) = {
        let (resolved_path, source) =
            resolver.find_include(context.entry(), context.current_file(), &path, path_tokens)?;
        let parsed = crate::parse::parse(&resolved_path.display().to_string(), &source)?;
        (resolved_path, parsed)
    };
    let eoi = tokens.pop().unwrap();
    tokens.push(Token::ending_newline());
    tokens.push(eoi);
    let mut tokenstream = tokens.into_iter().peekable();
    let current = context.current_file().clone();
    context.set_current_file(pathbuf.display().to_string());
    let output = root_preprocess(resolver, context, &mut tokenstream, false);
    context.set_current_file(current);
    output
}

fn directive_define_preprocess<R>(
    resolver: &mut R,
    context: &mut Context,
    tokenstream: &mut Peekable<impl Iterator<Item = Token>>,
) -> Result<(), Error>
where
    R: Resolver,
{
    let (ident_token, ident) = if let Some(token) = tokenstream.next() {
        match token.symbol() {
            Symbol::Word(ident) => {
                let ident = ident.to_string();
                (token, ident)
            }
            _ => {
                return Err(Error::ExpectedIdent {
                    token: Box::new(token.clone()),
                })
            }
        }
    } else {
        return Err(Error::UnexpectedEOF);
    };
    let mut skipped = false;
    if let Some(token) = tokenstream.peek() {
        if let Symbol::Whitespace(_) | Symbol::Comment(_) = token.symbol() {
            whitespace::skip(tokenstream);
            skipped = true;
        }
    }
    // check directive type
    if let Some(token) = tokenstream.peek() {
        match (token.symbol(), skipped) {
            (Symbol::LeftParenthesis, false) => {
                let args = read_args(resolver, context, tokenstream)?;
                whitespace::skip(tokenstream);
                if args.iter().any(|arg| arg.len() != 1) {
                    return Err(Error::DefineMultiTokenArgument {
                        token: Box::new(ident_token),
                    });
                }
                context.define(
                    ident,
                    ident_token,
                    Definition::Function(FunctionDefinition::new(
                        args.into_iter()
                            .map(|a| a.first().unwrap().clone())
                            .collect(),
                        directive_define_read_body(tokenstream),
                    )),
                )?;
            }
            (Symbol::Newline, _) => {
                context.define(ident, ident_token, Definition::Unit)?;
            }
            (_, _) => {
                context.define(
                    ident,
                    ident_token,
                    Definition::Value(directive_define_read_body(tokenstream)),
                )?;
                // return Err(Error::UnexpectedToken {
                //     token: Box::new(token.clone()),
                //     expected: vec![
                //         Symbol::LeftParenthesis,
                //         Symbol::Whitespace(Whitespace::Space),
                //         Symbol::Whitespace(Whitespace::Tab),
                //         Symbol::Escape,
                //     ],
                // });
            }
        }
    } else {
        return Err(Error::UnexpectedEOF);
    }
    Ok(())
}

fn directive_undef_preprocess(
    context: &mut Context,
    tokenstream: &mut Peekable<impl Iterator<Item = Token>>,
) -> Result<(), Error> {
    if let Some(token) = tokenstream.next() {
        match token.symbol() {
            Symbol::Word(ident) => {
                context.undefine(ident, &token)?;
                whitespace::skip(tokenstream);
                if let Symbol::Newline = tokenstream.peek().unwrap().symbol() {
                    tokenstream.next();
                } else {
                    return Err(Error::UnexpectedToken {
                        token: Box::new(tokenstream.next().unwrap()),
                        expected: vec![Symbol::Newline],
                    });
                }
            }
            _ => {
                return Err(Error::ExpectedIdent {
                    token: Box::new(token.clone()),
                })
            }
        }
    } else {
        return Err(Error::UnexpectedEOF);
    }
    Ok(())
}

fn directive_if_preprocess(
    context: &mut Context,
    tokenstream: &mut Peekable<impl Iterator<Item = Token>>,
) -> Result<(), Error> {
    let (ident_token, ident) = if let Some(token) = tokenstream.next() {
        match token.symbol() {
            Symbol::Word(ident) => {
                let ident = ident.to_string();
                (token, ident)
            }
            _ => {
                return Err(Error::ExpectedIdent {
                    token: Box::new(token.clone()),
                })
            }
        }
    } else {
        return Err(Error::UnexpectedEOF);
    };
    if let Some((_, definition)) = context.get(&ident, &ident_token) {
        if let Definition::Value(tokens) = definition {
            let read = [Symbol::Digit(1), Symbol::Word("1".to_string())]
                .contains(tokens.first().unwrap().symbol());
            context.ifstates_mut().push(if read {
                IfState::ReadingIf
            } else {
                IfState::PassingIf
            });
        } else {
            return Err(Error::IfUnitOrFunction {
                token: Box::new(ident_token),
            });
        }
    } else {
        return Err(Error::IfUndefined {
            token: Box::new(ident_token),
        });
    }
    eat_newline(tokenstream)
}

fn directive_ifdef_preprocess(
    context: &mut Context,
    tokenstream: &mut Peekable<impl Iterator<Item = Token>>,
    has: bool,
) -> Result<(), Error> {
    let (_, ident) = if let Some(token) = tokenstream.next() {
        match token.symbol() {
            Symbol::Word(ident) => {
                let ident = ident.to_string();
                (token, ident)
            }
            _ => {
                return Err(Error::ExpectedIdent {
                    token: Box::new(token.clone()),
                })
            }
        }
    } else {
        return Err(Error::UnexpectedEOF);
    };
    let has = context.has(&ident) == has;
    context.ifstates_mut().push(if has {
        IfState::ReadingIf
    } else {
        IfState::PassingIf
    });
    eat_newline(tokenstream)
}

fn directive_define_read_body(
    tokenstream: &mut Peekable<impl Iterator<Item = Token>>,
) -> Vec<Token> {
    let mut output: Vec<Token> = Vec::new();
    while let Some(token) = tokenstream.peek() {
        if let Symbol::Newline = token.symbol() {
            let builtin = Token::builtin();
            if output.last().unwrap_or(&builtin).symbol() == &Symbol::Escape {
                output.pop();
                output.push(tokenstream.next().unwrap());
            } else {
                tokenstream.next();
                break;
            }
        } else {
            output.push(tokenstream.next().unwrap());
        }
    }
    output
}

fn read_args<R>(
    resolver: &mut R,
    context: &mut Context,
    tokenstream: &mut Peekable<impl Iterator<Item = Token>>,
) -> Result<Vec<Vec<Token>>, Error>
where
    R: Resolver,
{
    let mut args = Vec::new();
    let mut arg: Vec<Token> = Vec::new();
    if let Some(token) = tokenstream.next() {
        match token.symbol() {
            Symbol::LeftParenthesis => {}
            _ => {
                return Err(Error::UnexpectedToken {
                    token: Box::new(token.clone()),
                    expected: vec![Symbol::LeftParenthesis],
                })
            }
        }
    } else {
        return Err(Error::UnexpectedEOF);
    }
    let mut depth = 0;
    while let Some(token) = tokenstream.peek() {
        match token.symbol() {
            Symbol::Comma => {
                tokenstream.next();
                while let Symbol::Whitespace(_) = arg.last().unwrap().symbol() {
                    arg.pop();
                }
                args.push(arg);
                arg = Vec::new();
                whitespace::skip(tokenstream);
            }
            Symbol::LeftParenthesis => {
                depth += 1;
                arg.push(tokenstream.next().unwrap());
                whitespace::skip(tokenstream);
            }
            Symbol::RightParenthesis => {
                if depth == 0 {
                    tokenstream.next();
                    if !arg.is_empty() {
                        while let Symbol::Whitespace(_) = arg.last().unwrap().symbol() {
                            arg.pop();
                        }
                    }
                    args.push(arg);
                    break;
                }
                depth -= 1;
                arg.push(tokenstream.next().unwrap());
            }
            Symbol::Word(word) => {
                if let Some((_source, definition)) = context.get(word, token) {
                    let token = token.clone();
                    tokenstream.next();
                    arg.append(&mut walk_definition(
                        resolver,
                        context,
                        tokenstream,
                        token,
                        definition,
                    )?);
                } else {
                    arg.push(tokenstream.next().unwrap());
                }
            }
            _ => {
                arg.push(tokenstream.next().unwrap());
            }
        }
    }
    Ok(args)
}

fn walk_line<R>(
    resolver: &mut R,
    context: &mut Context,
    tokenstream: &mut Peekable<impl Iterator<Item = Token>>,
) -> Result<Vec<Token>, Error>
where
    R: Resolver,
{
    let mut output = Vec::new();
    while let Some(token) = tokenstream.peek() {
        if let Symbol::Newline = token.symbol() {
            output.push(tokenstream.next().unwrap());
            break;
        }
        match token.symbol() {
            Symbol::Word(word) => {
                if let Some((_source, definition)) = context.get(word, token) {
                    let token = token.clone();
                    tokenstream.next();
                    output.append(&mut walk_definition(
                        resolver,
                        context,
                        tokenstream,
                        token,
                        definition,
                    )?);
                } else {
                    output.push(tokenstream.next().unwrap());
                }
            }
            Symbol::DoubleQuote => {
                output.push(tokenstream.next().unwrap());
                while let Some(token) = tokenstream.peek() {
                    if let Symbol::DoubleQuote = token.symbol() {
                        output.push(tokenstream.next().unwrap());
                        break;
                    }
                    output.push(tokenstream.next().unwrap());
                }
            }
            Symbol::Directive => {
                output.append(&mut directive_preprocess(
                    resolver,
                    context,
                    tokenstream,
                    true,
                )?);
            }
            _ => output.push(tokenstream.next().unwrap()),
        }
    }
    Ok(output)
}

fn walk_definition<R>(
    resolver: &mut R,
    context: &mut Context,
    tokenstream: &mut Peekable<impl Iterator<Item = Token>>,
    source: Token,
    definition: Definition,
) -> Result<Vec<Token>, Error>
where
    R: Resolver,
{
    let mut output = Vec::new();
    match definition {
        Definition::Value(tokens) => {
            let mut tokenstream = tokens.into_iter().peekable();
            while tokenstream.peek().is_some() {
                output.append(&mut root_preprocess(
                    resolver,
                    context,
                    &mut tokenstream,
                    true,
                )?);
            }
        }
        Definition::Function(func) => {
            let args = read_args(resolver, context, tokenstream)?;
            if args.len() != func.parameters().len() {
                return Err(Error::FunctionCallArgumentCount {
                    token: Box::new(source),
                    expected: func.parameters().len(),
                    got: args.len(),
                });
            }
            let mut stack = context.clone();
            for (param, arg) in func.parameters().iter().zip(args.into_iter()) {
                stack.define(
                    param.word().unwrap().to_string(),
                    param.clone(),
                    Definition::Value(root_preprocess(
                        resolver,
                        context,
                        &mut arg.into_iter().peekable(),
                        true,
                    )?),
                )?;
            }
            let mut tokenstream = func.body().iter().cloned().peekable();
            while tokenstream.peek().is_some() {
                output.append(&mut root_preprocess(
                    resolver,
                    &mut stack,
                    &mut tokenstream,
                    true,
                )?);
            }
        }
        Definition::Unit => {
            return Err(Error::ExpectedFunctionOrValue {
                token: Box::new(source),
            });
        }
    }
    Ok(output)
}

fn eat_newline(tokenstream: &mut Peekable<impl Iterator<Item = Token>>) -> Result<(), Error> {
    whitespace::skip(tokenstream);
    if let Some(token) = tokenstream.peek() {
        if let Symbol::Newline = token.symbol() {
            tokenstream.next();
        } else {
            return Err(Error::UnexpectedToken {
                token: Box::new(token.clone()),
                expected: vec![Symbol::Newline],
            });
        }
    } else {
        return Err(Error::UnexpectedEOF);
    }
    Ok(())
}
