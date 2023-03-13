#![deny(clippy::all, clippy::nursery, missing_docs)]
#![warn(clippy::pedantic)]

//! HEMTT - Arma 3 Preprocessor

use std::path::PathBuf;

use hemtt_tokens::whitespace;
use hemtt_tokens::{Symbol, Token};
use ifstate::IfState;

mod context;
mod defines;
mod error;
mod ifstate;
mod map;
mod parse;
mod resolver;

pub use context::{Context, Definition, FunctionDefinition};
pub use defines::{Defines, DefinitionLibrary};
pub use error::Error;
pub use map::{Mapping, Processed};
use peekmore::{PeekMore, PeekMoreIterator};
pub use resolver::resolvers;
pub use resolver::{
    resolvers::{LocalResolver, NoResolver},
    Resolver,
};

/// Preprocesses a config file.
///
/// # Errors
/// [`Error`]
///
/// # Panics
/// If the files
pub fn preprocess_file<R>(entry: &str, resolver: &R) -> Result<Vec<Token>, Error>
where
    R: Resolver,
{
    let mut context = Context::new(entry.to_string());
    let source = resolver.find_include(
        &context,
        PathBuf::from(entry).parent().unwrap().to_str().unwrap(),
        entry,
        PathBuf::from(entry).file_name().unwrap().to_str().unwrap(),
        vec![Token::builtin(None)],
    )?;
    let mut tokens = crate::parse::parse(entry, &source.1, &None)?;
    let eoi = tokens.pop().unwrap();
    tokens.push(Token::ending_newline(None));
    tokens.push(eoi);
    let mut tokenstream = tokens.into_iter().peekmore();
    root_preprocess(resolver, &mut context, &mut tokenstream, false)
}

/// # Errors
/// it can fail
pub fn preprocess_string(source: &str) -> Result<Vec<Token>, Error> {
    let tokens = crate::parse::parse("%anonymous%", source, &None)?;
    let mut context = Context::new(String::from("%anonymous%"));
    let mut tokenstream = tokens.into_iter().peekmore();
    root_preprocess(&NoResolver::new(), &mut context, &mut tokenstream, false)
}

fn root_preprocess<R>(
    resolver: &R,
    context: &mut Context,
    tokenstream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    allow_quote: bool,
) -> Result<Vec<Token>, Error>
where
    R: Resolver,
{
    let mut output = Vec::new();
    while let Some(token) = tokenstream.peek() {
        match token.symbol() {
            Symbol::Directive => {
                let token = token.clone();
                output.append(&mut directive_preprocess(
                    resolver,
                    context,
                    tokenstream,
                    allow_quote,
                    token,
                )?);
            }
            Symbol::Comment(_) | Symbol::Whitespace(_) => {
                tokenstream.next();
            }
            Symbol::Slash => {
                if matches!(
                    tokenstream.peek_forward(1).map(Token::symbol),
                    Some(Symbol::Slash)
                ) {
                    whitespace::skip_comment(tokenstream);
                } else if context.ifstates().reading() {
                    output.push(tokenstream.next().unwrap());
                }
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

#[allow(clippy::too_many_lines)]
fn directive_preprocess<R>(
    resolver: &R,
    context: &mut Context,
    tokenstream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    allow_quote: bool,
    from: Token,
) -> Result<Vec<Token>, Error>
where
    R: Resolver,
{
    let Some(token) = tokenstream.peek() else {
        return Err(Error::UnexpectedEOF {
            token: Box::new(from),
        });
    };
    let token = token.clone();
    match token.symbol() {
        Symbol::Directive => {}
        _ => {
            return Err(Error::UnexpectedToken {
                token: Box::new(token.clone()),
                expected: vec![Symbol::Directive],
                trace: context.trace(),
            })
        }
    }
    let mut output = Vec::new();
    tokenstream.next();
    if let Some(token) = tokenstream.next() {
        if let Symbol::Word(command) = token.symbol() {
            match (command.as_str(), context.ifstates().reading()) {
                ("include", true) => {
                    whitespace::skip(tokenstream);
                    context.push(token.clone());
                    output.append(&mut directive_include_preprocess(
                        resolver,
                        context,
                        tokenstream,
                        token,
                    )?);
                    context.pop();
                }
                ("define", true) => {
                    whitespace::skip(tokenstream);
                    directive_define_preprocess(resolver, context, tokenstream, token)?;
                }
                ("undef", true) => {
                    whitespace::skip(tokenstream);
                    directive_undef_preprocess(context, tokenstream, token)?;
                }
                ("if", true) => {
                    whitespace::skip(tokenstream);
                    directive_if_preprocess(context, tokenstream, token)?;
                }
                ("ifdef", true) => {
                    whitespace::skip(tokenstream);
                    directive_ifdef_preprocess(context, tokenstream, true, token)?;
                }
                ("ifndef", true) => {
                    whitespace::skip(tokenstream);
                    directive_ifdef_preprocess(context, tokenstream, false, token)?;
                }
                ("ifdef" | "ifndef", false) => {
                    context.ifstates_mut().push(IfState::PassingChild);
                    whitespace::skip(tokenstream);
                    tokenstream.next();
                    eat_newline(tokenstream, context, &token)?;
                }
                ("else", _) => {
                    context.ifstates_mut().flip();
                    eat_newline(tokenstream, context, &token)?;
                }
                ("endif", _) => {
                    context.ifstates_mut().pop();
                    eat_newline(tokenstream, context, &token)?;
                }
                (_, true) => {
                    if allow_quote {
                        let source = token.source().clone();
                        output.push(Token::new(
                            Symbol::DoubleQuote,
                            source.clone(),
                            Some(Box::new(token.clone())),
                        ));
                        if let Symbol::Word(word) = token.symbol() {
                            if let Some((_source, definition)) = context.get(word, &token) {
                                output.append(
                                    &mut walk_definition(
                                        resolver,
                                        context,
                                        tokenstream,
                                        token.clone(),
                                        definition,
                                    )?
                                    .into_iter()
                                    .filter(|t| t.symbol() != &Symbol::Join)
                                    .collect(),
                                );
                            } else {
                                output.push(token.clone());
                            }
                        } else {
                            output.push(token.clone());
                        }
                        output.push(Token::new(
                            Symbol::DoubleQuote,
                            source,
                            Some(Box::new(token)),
                        ));
                    } else {
                        return Err(Error::UnknownDirective {
                            directive: Box::new(token),
                            trace: context.trace(),
                        });
                    }
                }
                _ => {}
            }
        }
    } else if !allow_quote {
        return Err(Error::UnexpectedEOF {
            token: Box::new(from),
        });
    } else {
        output.push(token);
    }
    Ok(output)
}

fn directive_include_preprocess<R>(
    resolver: &R,
    context: &mut Context,
    tokenstream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    from: Token,
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
                trace: context.trace(),
            })
        }
    };
    let mut path = String::new();
    let mut path_tokens = Vec::new();
    let mut last = None;
    while let Some(token) = tokenstream.peek() {
        if token.symbol() == &encased_in {
            tokenstream.next();
            break;
        }
        if token.symbol() == &Symbol::Newline {
            return Err(Error::UnexpectedToken {
                token: Box::new(token.clone()),
                expected: vec![encased_in],
                trace: context.trace(),
            });
        }
        path.push_str(token.to_string().as_str());
        path_tokens.push(token.clone());
        last = tokenstream.next();
    }
    if tokenstream.peek().is_none() {
        return Err(Error::UnexpectedEOF {
            token: Box::new(last.unwrap_or_else(|| from.clone())),
        });
    }
    let (pathbuf, mut tokens) = {
        let (resolved_path, source) = resolver.find_include(
            context,
            context.entry(),
            context.current_file(),
            &path,
            path_tokens,
        )?;
        let parsed = crate::parse::parse(
            &resolved_path.display().to_string(),
            &source,
            &Some(Box::new(from)),
        )?;
        (resolved_path, parsed)
    };
    // Remove EOI token
    tokens.pop().unwrap();
    tokens.push(Token::ending_newline(None));
    let mut tokenstream = tokens.into_iter().peekmore();
    let current = context.current_file().clone();
    context.set_current_file(pathbuf.display().to_string());
    let output = root_preprocess(resolver, context, &mut tokenstream, false);
    context.set_current_file(current);
    output
}

fn directive_define_preprocess<R>(
    resolver: &R,
    context: &mut Context,
    tokenstream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    from: Token,
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
                    token: Box::new(token),
                    trace: context.trace(),
                })
            }
        }
    } else {
        return Err(Error::UnexpectedEOF {
            token: Box::new(from),
        });
    };
    let mut skipped = false;
    let mut last = None;
    if let Some(token) = tokenstream.peek() {
        if let Symbol::Whitespace(_) | Symbol::Comment(_) = token.symbol() {
            last = whitespace::skip(tokenstream);
            skipped = true;
        }
    }
    // check directive type
    if let Some(token) = tokenstream.peek() {
        match (token.symbol(), skipped) {
            (Symbol::LeftParenthesis, false) => {
                let token = token.clone();
                let args = read_args(resolver, context, tokenstream, &token, false)?;
                whitespace::skip(tokenstream);
                if args.iter().any(|arg| arg.len() != 1) {
                    return Err(Error::DefineMultiTokenArgument {
                        token: Box::new(ident_token),
                        trace: context.trace(),
                    });
                }
                let def = FunctionDefinition::new(
                    args.into_iter()
                        .map(|a| a.first().unwrap().clone())
                        .collect(),
                    directive_define_read_body(tokenstream),
                );
                context.define(ident, ident_token, Definition::Function(def))?;
            }
            (Symbol::Newline, _) => {
                context.define(ident, ident_token, Definition::Unit)?;
            }
            (_, _) => {
                let val = directive_define_read_body(tokenstream);
                context.define(ident, ident_token, Definition::Value(val))?;
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
        return Err(Error::UnexpectedEOF {
            token: Box::new(last.unwrap_or_else(|| from.clone())),
        });
    }
    Ok(())
}

fn directive_undef_preprocess(
    context: &mut Context,
    tokenstream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    from: Token,
) -> Result<(), Error> {
    if let Some(token) = tokenstream.next() {
        match token.symbol() {
            Symbol::Word(ident) => {
                context.undefine(ident, &token)?;
                whitespace::skip(tokenstream);
                if matches!(tokenstream.peek().unwrap().symbol(), Symbol::Newline) {
                    tokenstream.next();
                } else {
                    return Err(Error::UnexpectedToken {
                        token: Box::new(tokenstream.next().unwrap()),
                        expected: vec![Symbol::Newline],
                        trace: context.trace(),
                    });
                }
            }
            _ => {
                return Err(Error::ExpectedIdent {
                    token: Box::new(token.clone()),
                    trace: context.trace(),
                })
            }
        }
    } else {
        return Err(Error::UnexpectedEOF {
            token: Box::new(from),
        });
    }
    Ok(())
}

fn directive_if_preprocess(
    context: &mut Context,
    tokenstream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    from: Token,
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
                    trace: context.trace(),
                })
            }
        }
    } else {
        return Err(Error::UnexpectedEOF {
            token: Box::new(from),
        });
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
                trace: context.trace(),
            });
        }
    } else {
        return Err(Error::IfUndefined {
            token: Box::new(ident_token),
            trace: context.trace(),
        });
    }
    eat_newline(tokenstream, context, &ident_token)
}

fn directive_ifdef_preprocess(
    context: &mut Context,
    tokenstream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    has: bool,
    from: Token,
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
                    trace: context.trace(),
                })
            }
        }
    } else {
        return Err(Error::UnexpectedEOF {
            token: Box::new(from),
        });
    };
    let has = context.has(&ident) == has;
    context.ifstates_mut().push(if has {
        IfState::ReadingIf
    } else {
        IfState::PassingIf
    });
    eat_newline(tokenstream, context, &ident_token)
}

fn directive_define_read_body(
    tokenstream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
) -> Vec<Token> {
    let mut output: Vec<Token> = Vec::new();
    while let Some(token) = tokenstream.peek() {
        if matches!(token.symbol(), Symbol::Newline) {
            let builtin = Token::builtin(Some(Box::new(token.clone())));
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

#[allow(clippy::too_many_lines)]
fn read_args<R>(
    resolver: &R,
    context: &mut Context,
    tokenstream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    from: &Token,
    recursive: bool,
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
                    trace: context.trace(),
                })
            }
        }
    } else {
        return Err(Error::UnexpectedEOF {
            token: Box::new(from.clone()),
        });
    }
    let mut depth = 0;
    let mut quote = false;
    while let Some(token) = tokenstream.peek() {
        match token.symbol() {
            Symbol::DoubleQuote => {
                quote = !quote;
                arg.push(tokenstream.next().unwrap());
            }
            Symbol::Comma => {
                if quote {
                    arg.push(tokenstream.next().unwrap());
                    continue;
                }
                tokenstream.next();
                while let Symbol::Whitespace(_) = arg.last().unwrap().symbol() {
                    arg.pop();
                }
                args.push(arg);
                arg = Vec::new();
                whitespace::skip(tokenstream);
            }
            Symbol::LeftParenthesis => {
                if quote {
                    arg.push(tokenstream.next().unwrap());
                    continue;
                }
                depth += 1;
                arg.push(tokenstream.next().unwrap());
                whitespace::skip(tokenstream);
            }
            Symbol::RightParenthesis => {
                if quote {
                    arg.push(tokenstream.next().unwrap());
                    continue;
                }
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
                if quote {
                    arg.push(tokenstream.next().unwrap());
                    continue;
                }
                if recursive {
                    if let Some((_source, definition)) = context.get(word, token) {
                        let token = token.clone();
                        tokenstream.next();
                        if definition.is_function()
                            && tokenstream.peek().unwrap().symbol() != &Symbol::LeftParenthesis
                        {
                            arg.push(tokenstream.next().unwrap());
                            continue;
                        }
                        arg.append(&mut walk_definition(
                            resolver,
                            context,
                            tokenstream,
                            token,
                            definition,
                        )?);
                        continue;
                    }
                }
                arg.push(tokenstream.next().unwrap());
            }
            Symbol::Newline => {
                let builtin = Token::builtin(Some(Box::new(token.clone())));
                if arg.last().unwrap_or(&builtin).symbol() == &Symbol::Escape {
                    arg.pop();
                }
                arg.push(tokenstream.next().unwrap());
            }
            _ => {
                arg.push(tokenstream.next().unwrap());
            }
        }
    }
    Ok(args)
}

fn walk_line<R>(
    resolver: &R,
    context: &mut Context,
    tokenstream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
) -> Result<Vec<Token>, Error>
where
    R: Resolver,
{
    let mut output = Vec::new();
    while let Some(token) = tokenstream.peek() {
        if matches!(token.symbol(), Symbol::Newline) {
            // check if last token was an escape
            let builtin = Token::builtin(Some(Box::new(token.clone())));
            if output.last().unwrap_or(&builtin).symbol() == &Symbol::Escape {
                output.pop();
                tokenstream.next();
            } else {
                output.push(tokenstream.next().unwrap());
            }
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
                    if matches!(token.symbol(), Symbol::DoubleQuote) {
                        output.push(tokenstream.next().unwrap());
                        break;
                    }
                    output.push(tokenstream.next().unwrap());
                }
            }
            Symbol::Directive => {
                let token = token.clone();
                output.append(&mut directive_preprocess(
                    resolver,
                    context,
                    tokenstream,
                    true,
                    token,
                )?);
            }
            Symbol::Slash => {
                if matches!(
                    tokenstream.peek_forward(1).map(Token::symbol),
                    Some(Symbol::Slash)
                ) {
                    whitespace::skip_comment(tokenstream);
                } else {
                    tokenstream.move_cursor_back().unwrap();
                    output.push(tokenstream.next().unwrap());
                }
            }
            _ => output.push(tokenstream.next().unwrap()),
        }
    }
    Ok(output)
}

fn walk_definition<R>(
    resolver: &R,
    context: &mut Context,
    tokenstream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    from: Token,
    definition: Definition,
) -> Result<Vec<Token>, Error>
where
    R: Resolver,
{
    let mut output = Vec::new();
    match definition {
        Definition::Value(tokens) => {
            let parent = Some(Box::new(from));
            let mut tokenstream = tokens
                .into_iter()
                .map(|mut t| {
                    t.set_parent(parent.clone());
                    t
                })
                .collect::<Vec<_>>()
                .into_iter()
                .peekmore();
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
            let args = read_args(resolver, context, tokenstream, &from, true)?;
            if args.len() != func.parameters().len() {
                return Err(Error::FunctionCallArgumentCount {
                    token: Box::new(from),
                    expected: func.parameters().len(),
                    got: args.len(),
                    trace: context.trace(),
                    defines: context.definitions().clone(),
                });
            }
            let mut stack = context.stack(from.clone());
            for (param, arg) in func.parameters().iter().zip(args.into_iter()) {
                let def = Definition::Value(root_preprocess(
                    resolver,
                    &mut stack,
                    &mut arg.into_iter().peekmore(),
                    true,
                )?);
                stack.define(param.word().unwrap().to_string(), param.clone(), def)?;
            }
            let parent = Some(Box::new(from));
            let mut tokenstream = func
                .body()
                .iter()
                .cloned()
                .map(|mut t| {
                    t.set_parent(parent.clone());
                    t
                })
                .collect::<Vec<_>>()
                .into_iter()
                .peekmore();
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
                token: Box::new(from),
                trace: context.trace(),
            });
        }
    }
    Ok(output)
}

fn eat_newline(
    tokenstream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    context: &mut Context,
    from: &Token,
) -> Result<(), Error> {
    let last = whitespace::skip(tokenstream);
    if let Some(token) = tokenstream.peek() {
        if matches!(token.symbol(), Symbol::Newline) {
            tokenstream.next();
        } else {
            return Err(Error::UnexpectedToken {
                token: Box::new(token.clone()),
                expected: vec![Symbol::Newline],
                trace: context.trace(),
            });
        }
    } else {
        return Err(Error::UnexpectedEOF {
            token: Box::new(last.unwrap_or_else(|| from.clone())),
        });
    }
    Ok(())
}
