#![deny(clippy::all, clippy::nursery, missing_docs)]
#![warn(clippy::pedantic)]

//! HEMTT - Arma 3 Preprocessor

use codes::{
    pe10_function_as_value::FunctionAsValue,
    pe11_expected_funtion_or_value::ExpectedFunctionOrValue,
    pe12_include_not_found::IncludeNotFound, pe13_include_not_encased::IncludeNotEncased,
    pe1_unexpected_token::UnexpectedToken, pe2_unexpected_eof::UnexpectedEOF,
    pe3_expected_ident::ExpectedIdent, pe5_define_multitoken_argument::DefineMultiTokenArgument,
    pe7_if_unit_or_function::IfUnitOrFunction, pe8_if_undefined::IfUndefined,
    pe9_function_call_argument_count::FunctionCallArgumentCount,
};
use hemtt_error::{
    processed::Processed,
    tokens::{Symbol, Token},
};
use ifstate::IfState;

pub mod codes;
mod context;
mod defines;
mod error;
mod ifstate;
mod parse;
mod resolver;

pub use context::{Context, Definition, FunctionDefinition};
pub use defines::{Defines, DefinitionLibrary};
pub use error::Error;
pub use parse::parse;
use peekmore::{PeekMore, PeekMoreIterator};
pub use resolver::Resolver;
use tracing::warn;
use vfs::VfsPath;

/// Preprocesses a config file.
///
/// # Errors
/// [`Error`]
///
/// # Panics
/// If the files
pub fn preprocess_file(entry: &VfsPath, resolver: &Resolver) -> Result<Processed, Error> {
    let mut context = Context::new(entry.clone());
    let source = entry.read_to_string()?;
    let mut tokens = crate::parse::parse(entry, &source, &None)?;
    let eoi = tokens.pop().unwrap();
    tokens.push(Token::ending_newline(None));
    tokens.push(eoi);
    let mut tokenstream = tokens.into_iter().peekmore();
    let processed = root_preprocess(resolver, &mut context, &mut tokenstream, false)?;
    let warnings = context.warnings().map_or_else(
        || {
            warn!("context was not resolved");
            Vec::new()
        },
        |warnings| warnings,
    );
    Ok(Processed::from_tokens(processed, warnings))
}

fn root_preprocess(
    resolver: &Resolver,
    context: &mut Context,
    tokenstream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    allow_quote: bool,
) -> Result<Vec<Token>, Error> {
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
                    skip_comment(tokenstream);
                } else if context.ifstates().reading() {
                    output.push(tokenstream.next().unwrap());
                }
            }
            _ => {
                if context.ifstates().reading() {
                    output.append(&mut walk_line(resolver, context, tokenstream, allow_quote)?);
                } else {
                    tokenstream.next();
                }
            }
        }
    }
    Ok(output)
}

#[allow(clippy::too_many_lines)]
fn directive_preprocess(
    resolver: &Resolver,
    context: &mut Context,
    tokenstream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    allow_quote: bool,
    from: Token,
) -> Result<Vec<Token>, Error> {
    let Some(token) = tokenstream.peek() else {
        return Err(Error::Code(Box::new(UnexpectedEOF {
            token: Box::new(from),
        })));
    };
    let directive_token = token.clone();
    if directive_token.symbol() != &Symbol::Directive {
        return Err(Error::Code(Box::new(UnexpectedToken {
            token: Box::new(directive_token),
            expected: vec![Symbol::Directive],
            trace: context.trace(),
        })));
    }
    let mut output = Vec::new();
    tokenstream.next();
    if let Some(token) = tokenstream.next() {
        if let Symbol::Word(command) = token.symbol() {
            match (command.as_str(), context.ifstates().reading()) {
                ("include", true) => {
                    skip(tokenstream);
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
                    skip(tokenstream);
                    directive_define_preprocess(resolver, context, tokenstream, token)?;
                }
                ("undef", true) => {
                    skip(tokenstream);
                    directive_undef_preprocess(context, tokenstream, token)?;
                }
                ("if", true) => {
                    skip(tokenstream);
                    directive_if_preprocess(context, tokenstream, token)?;
                }
                ("ifdef", true) => {
                    skip(tokenstream);
                    directive_ifdef_preprocess(context, tokenstream, true, token)?;
                }
                ("ifndef", true) => {
                    skip(tokenstream);
                    directive_ifdef_preprocess(context, tokenstream, false, token)?;
                }
                ("ifdef" | "ifndef", false) => {
                    context.ifstates_mut().push(IfState::PassingChild);
                    skip(tokenstream);
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
                            if let Some((source, definition)) = context.get(word, &token) {
                                output.append(
                                    &mut walk_definition(
                                        resolver,
                                        context,
                                        tokenstream,
                                        token.clone(),
                                        definition,
                                        &source,
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
                        output.push(directive_token);
                        output.push(token);
                    }
                }
                _ => {}
            }
        } else if context.ifstates().reading() {
            output.push(directive_token);
            output.push(token);
        }
    } else if !allow_quote {
        return Err(Error::Code(Box::new(UnexpectedEOF {
            token: Box::new(from),
        })));
    } else {
        output.push(directive_token);
    }
    Ok(output)
}

fn directive_include_preprocess(
    resolver: &Resolver,
    context: &mut Context,
    tokenstream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    from: Token,
) -> Result<Vec<Token>, Error> {
    let encased_in = match tokenstream.peek().unwrap().symbol() {
        Symbol::DoubleQuote | Symbol::SingleQuote => tokenstream.next().unwrap().symbol().clone(),
        Symbol::LeftAngle => {
            tokenstream.next();
            Symbol::RightAngle
        }
        _ => {
            return Err(Error::Code(Box::new(IncludeNotEncased {
                token: Box::new(tokenstream.peek().unwrap().clone()),
                trace: context.trace(),
            })))
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
            return Err(Error::Code(Box::new(UnexpectedToken {
                token: Box::new(token.clone()),
                expected: vec![encased_in],
                trace: context.trace(),
            })));
        }
        path.push_str(token.to_string().as_str());
        path_tokens.push(token.clone());
        last = tokenstream.next();
    }
    if tokenstream.peek().is_none() {
        return Err(Error::Code(Box::new(UnexpectedEOF {
            token: Box::new(last.unwrap_or_else(|| from.clone())),
        })));
    }
    let (vfs, mut tokens) = {
        let Ok(Some((resolved_path, source))) = resolver.find_include(
            context.current_file(),
            &path,
        ) else {
            return Err(Error::Code(Box::new(IncludeNotFound { token: path_tokens, trace: context.trace() })))
        };
        let parsed = crate::parse::parse(&resolved_path, &source, &Some(Box::new(from)))?;
        (resolved_path, parsed)
    };
    // Remove EOI token
    tokens.pop().unwrap();
    tokens.push(Token::ending_newline(None));
    let mut tokenstream = tokens.into_iter().peekmore();
    let current = context.current_file().clone();
    context.set_current_file(vfs);
    let output = root_preprocess(resolver, context, &mut tokenstream, false);
    context.set_current_file(current);
    output
}

fn directive_define_preprocess(
    resolver: &Resolver,
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
                return Err(Error::Code(Box::new(ExpectedIdent {
                    token: Box::new(token),
                    trace: context.trace(),
                })))
            }
        }
    } else {
        return Err(Error::Code(Box::new(UnexpectedEOF {
            token: Box::new(from),
        })));
    };
    let mut skipped = Vec::new();
    if let Some(token) = tokenstream.peek() {
        if let Symbol::Whitespace(_) | Symbol::Comment(_) = token.symbol() {
            skipped = skip(tokenstream);
        }
    }
    // check directive type
    if let Some(token) = tokenstream.peek() {
        match (token.symbol(), !skipped.is_empty()) {
            (Symbol::LeftParenthesis, false) => {
                let token = token.clone();
                let args = read_args(resolver, context, tokenstream, &token, false, &ident_token)?;
                skip(tokenstream);
                if args.iter().any(|arg| arg.len() != 1) {
                    return Err(Error::Code(Box::new(DefineMultiTokenArgument {
                        token: Box::new(ident_token),
                        trace: context.trace(),
                        arguments: args,
                    })));
                }
                let def = FunctionDefinition::new(
                    args.into_iter()
                        .map(|a| a.first().unwrap().clone())
                        .collect(),
                    directive_define_read_body(tokenstream),
                );
                context.define(ident, ident_token, Definition::Function(def), false)?;
            }
            (Symbol::Newline, _) => {
                context.define(ident, ident_token, Definition::Unit(skipped), false)?;
            }
            (_, _) => {
                let val = directive_define_read_body(tokenstream);
                context.define(ident, ident_token, Definition::Value(val), false)?;
            }
        }
    } else {
        let last = skipped.last().cloned();
        return Err(Error::Code(Box::new(UnexpectedEOF {
            token: Box::new(last.unwrap_or_else(|| from.clone())),
        })));
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
                skip(tokenstream);
                if matches!(tokenstream.peek().unwrap().symbol(), Symbol::Newline) {
                    tokenstream.next();
                } else {
                    return Err(Error::Code(Box::new(UnexpectedToken {
                        token: Box::new(tokenstream.next().unwrap()),
                        expected: vec![Symbol::Newline],
                        trace: context.trace(),
                    })));
                }
            }
            _ => {
                return Err(Error::Code(Box::new(ExpectedIdent {
                    token: Box::new(token.clone()),
                    trace: context.trace(),
                })))
            }
        }
    } else {
        return Err(Error::Code(Box::new(UnexpectedEOF {
            token: Box::new(from),
        })));
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
                return Err(Error::Code(Box::new(ExpectedIdent {
                    token: Box::new(token.clone()),
                    trace: context.trace(),
                })))
            }
        }
    } else {
        return Err(Error::Code(Box::new(UnexpectedEOF {
            token: Box::new(from),
        })));
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
            return Err(Error::Code(Box::new(IfUnitOrFunction {
                token: Box::new(ident_token),
                trace: context.trace(),
                defines: context.definitions().clone(),
            })));
        }
    } else {
        return Err(Error::Code(Box::new(IfUndefined {
            token: Box::new(ident_token),
            trace: context.trace(),
            defines: context.definitions().clone(),
        })));
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
                return Err(Error::Code(Box::new(ExpectedIdent {
                    token: Box::new(token.clone()),
                    trace: context.trace(),
                })))
            }
        }
    } else {
        return Err(Error::Code(Box::new(UnexpectedEOF {
            token: Box::new(from),
        })));
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
fn read_args(
    resolver: &Resolver,
    context: &mut Context,
    tokenstream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    from: &Token,
    recursive: bool,
    source: &Token,
) -> Result<Vec<Vec<Token>>, Error> {
    let mut args = Vec::new();
    let mut arg: Vec<Token> = Vec::new();
    if let Some(token) = tokenstream.next() {
        match token.symbol() {
            Symbol::LeftParenthesis => {}
            _ => {
                return Err(Error::Code(Box::new(FunctionAsValue {
                    token: Box::new(token.clone()),
                    source: Box::new(source.clone()),
                    trace: context.trace(),
                })))
            }
        }
    } else {
        return Err(Error::Code(Box::new(UnexpectedEOF {
            token: Box::new(from.clone()),
        })));
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
                skip(tokenstream);
            }
            Symbol::LeftParenthesis => {
                if quote {
                    arg.push(tokenstream.next().unwrap());
                    continue;
                }
                depth += 1;
                arg.push(tokenstream.next().unwrap());
                skip(tokenstream);
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
                    if let Some((source, definition)) = context.get(word, token) {
                        let token = token.clone();
                        tokenstream.next();
                        if definition.is_function()
                            && tokenstream.peek().unwrap().symbol() != &Symbol::LeftParenthesis
                        {
                            arg.push(tokenstream.next().unwrap());
                            continue;
                        }
                        context.push(source.clone());
                        arg.append(&mut walk_definition(
                            resolver,
                            context,
                            tokenstream,
                            token,
                            definition,
                            &source,
                        )?);
                        context.pop();
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

fn walk_line(
    resolver: &Resolver,
    context: &mut Context,
    tokenstream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    // Allow quotes when inside of a macro
    allow_quote: bool,
) -> Result<Vec<Token>, Error> {
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
                if let Some((source, definition)) = context.get(word, token) {
                    let token = token.clone();
                    context.push(source.clone());
                    tokenstream.next();
                    output.append(&mut walk_definition(
                        resolver,
                        context,
                        tokenstream,
                        token,
                        definition,
                        &source,
                    )?);
                    context.pop();
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
                    allow_quote,
                    token,
                )?);
            }
            Symbol::Slash => {
                if matches!(
                    tokenstream.peek_forward(1).map(Token::symbol),
                    Some(Symbol::Slash)
                ) {
                    skip_comment(tokenstream);
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

fn walk_definition(
    resolver: &Resolver,
    context: &mut Context,
    tokenstream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    from: Token,
    definition: Definition,
    source: &Token,
) -> Result<Vec<Token>, Error> {
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
            let args = read_args(resolver, context, tokenstream, &from, true, source)?;
            if args.len() != func.parameters().len() {
                return Err(Error::Code(Box::new(FunctionCallArgumentCount {
                    token: Box::new(from),
                    expected: func.parameters().len(),
                    got: args.len(),
                    trace: context.trace(),
                    defines: context.definitions().clone(),
                })));
            }
            let mut stack = context.stack(from.clone());
            for (param, arg) in func.parameters().iter().zip(args.into_iter()) {
                let def = Definition::Value(root_preprocess(
                    resolver,
                    &mut stack,
                    &mut arg.into_iter().peekmore(),
                    true,
                )?);
                stack.define(param.word().unwrap().to_string(), param.clone(), def, true)?;
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
        Definition::Unit(skipped) => {
            return Err(Error::Code(Box::new(ExpectedFunctionOrValue {
                token: Box::new(from),
                trace: context.trace(),
                skipped,
            })));
        }
    }
    Ok(output)
}

fn eat_newline(
    tokenstream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    context: &mut Context,
    from: &Token,
) -> Result<(), Error> {
    let skipped = skip(tokenstream);
    if let Some(token) = tokenstream.peek() {
        if matches!(token.symbol(), Symbol::Newline) {
            tokenstream.next();
        } else {
            return Err(Error::Code(Box::new(UnexpectedToken {
                token: Box::new(token.clone()),
                expected: vec![Symbol::Newline],
                trace: context.trace(),
            })));
        }
    } else {
        let last = skipped.last().cloned();
        return Err(Error::Code(Box::new(UnexpectedEOF {
            token: Box::new(last.unwrap_or_else(|| from.clone())),
        })));
    }
    Ok(())
}

/// Skip through whitespace
fn skip(input: &mut PeekMoreIterator<impl Iterator<Item = Token>>) -> Vec<Token> {
    let mut skipped = Vec::new();
    while let Some(token) = input.peek() {
        if token.symbol().is_whitespace() {
            if let Some(token) = input.next() {
                skipped.push(token);
            }
        } else if token.symbol() == &Symbol::Slash {
            if let Some(next_token) = input.peek_forward(1) {
                if next_token.symbol() == &Symbol::Slash {
                    skipped.extend(skip_comment(input));
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }
    skipped
}

/// Skip through a comment until a newline is found
/// Assumes the slashes are peeked but not consumed
fn skip_comment(input: &mut PeekMoreIterator<impl Iterator<Item = Token>>) -> Vec<Token> {
    let mut skipped = Vec::new();
    if let Some(token) = input.next() {
        skipped.push(token);
    }
    if let Some(token) = input.next() {
        skipped.push(token);
    }
    while let Some(token) = input.peek() {
        if token.symbol() == &Symbol::Newline {
            break;
        }
        if let Some(token) = input.next() {
            skipped.push(token);
        }
    }
    skipped
}
