use std::collections::HashMap;
use std::iter::Peekable;
use std::vec::IntoIter;

use pest::error::Error;
use pest::Parser;

mod token;
#[cfg(test)]
use token::Whitespace;
use token::{PreProcessParser, Rule, Token, TokenPos};

mod render;
pub use render::render;

mod define;
use define::Define;
mod ifstate;
use ifstate::{IfState, IfStates};

pub fn tokenize(source: &str, path: &str) -> Result<Vec<TokenPos>, Error<Rule>> {
    let mut tokens = Vec::new();

    let pairs = PreProcessParser::parse(Rule::file, source)?;
    for pair in pairs {
        tokens.push(TokenPos::new(path, pair))
    }

    Ok(tokens)
}

macro_rules! skip_whitespace {
    ($i: ident) => {{
        let mut next = $i.peek();
        loop {
            if let Some(tp) = next {
                if tp.token().is_whitespace() {
                    $i.next();
                    next = $i.peek();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }};
}

macro_rules! read_args {
    ($i: ident) => {{
        let mut ret: Vec<Vec<TokenPos>> = Vec::new();
        let mut next = $i.next();
        let mut arg: Vec<TokenPos> = Vec::new();
        let mut level = 0;
        if let Some(ref tp) = next {
            if let Token::LeftParenthesis = tp.token() {
                next = $i.next();
            }
        }
        loop {
            if let Some(tp) = next {
                match tp.token() {
                    Token::LeftParenthesis => {
                        level += 1;
                        arg.push(TokenPos::anon(Token::LeftParenthesis));
                    }
                    Token::RightParenthesis => {
                        if level == 0 {
                            if !arg.is_empty() {
                                ret.push(arg);
                            }
                            break;
                        } else {
                            arg.push(TokenPos::anon(Token::RightParenthesis));
                        }
                        level -= 1;
                    }
                    Token::Comma => {
                        if level == 0 {
                            if !arg.is_empty() {
                                ret.push(arg);
                                arg = Vec::new();
                            }
                        } else {
                            arg.push(TokenPos::anon(Token::Comma));
                        }
                    }
                    _ => arg.push(tp),
                }
            } else {
                break;
            }
            next = $i.next();
        }
        ret
    }};
}

#[test]
fn test_read_args() {
    let tokens = tokenize("(B(C); call f);", "").unwrap();
    let mut a = tokens.into_iter().peekable();
    assert_eq!(
        vec![vec![
            Token::Word(String::from("B")),
            Token::LeftParenthesis,
            Token::Word(String::from("C")),
            Token::RightParenthesis,
            Token::Semicolon,
            Token::Whitespace(Whitespace::Space),
            Token::Word(String::from("call")),
            Token::Whitespace(Whitespace::Space),
            Token::Word(String::from("f"))
        ]],
        vec![read_args!(a)[0]
            .iter()
            .map(|tp| tp.token().to_owned())
            .collect::<Vec<Token>>()]
    )
}

macro_rules! read_line {
    ($i: ident) => {{
        let mut ret: Vec<TokenPos> = Vec::new();
        let mut next = $i.next();
        // Skip initial whitespace
        loop {
            if let Some(tp) = &next {
                if tp.token().is_whitespace() {
                    next = $i.next();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        loop {
            if let Some(tp) = next {
                match tp.token() {
                    Token::Newline => break,
                    Token::Escape => {
                        if let Some(tp) = $i.peek() {
                            if tp.token() == &Token::Newline {
                                $i.next();
                            }
                        }
                        next = $i.next();
                        loop {
                            if let Some(ref tp) = next {
                                if tp.token().is_whitespace() {
                                    next = $i.next();
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                    _ => {
                        ret.push(tp);
                        next = $i.next();
                    }
                }
            } else {
                break;
            }
        }
        ret
    }};
}

#[test]
fn test_read_line() {
    let tokens = tokenize("test = false;\ntest = true;\n", "").unwrap();
    let mut a = tokens.into_iter().peekable();
    assert_eq!(
        vec![
            Token::Word(String::from("test")),
            Token::Whitespace(Whitespace::Space),
            Token::Assignment,
            Token::Whitespace(Whitespace::Space),
            Token::Word(String::from("false")),
            Token::Semicolon,
        ],
        read_line!(a)
            .iter()
            .map(|tp| tp.token().to_owned())
            .collect::<Vec<Token>>()
    )
}

pub fn _resolve<R>(
    ident: &str,
    define: &Define,
    resolver: R,
    defines: &HashMap<String, Define>,
) -> Option<Vec<TokenPos>>
where
    R: Fn(&str) -> String + Copy,
{
    if let Some(d) = defines.get(ident) {
        let mut ret = Vec::new();
        let mut context = defines.to_owned();
        if let Some(dargs) = &d.args {
            if let Some(args) = &define.args {
                if dargs.len() != args.len() {
                    panic!("Invalid arg lengths");
                }
                for i in 0..dargs.len() {
                    if let Token::Word(key) = &dargs[i][0].token() {
                        if args[i].len() == 1 {
                            if let Token::Word(value) = &args[i][0].token() {
                                context.insert(
                                    key.to_owned(),
                                    if let Some(ed) = defines.get(value) {
                                        ed.to_owned()
                                    } else {
                                        Define {
                                            args: None,
                                            statement: vec![args[i][0].to_owned()],
                                            call: false,
                                        }
                                    },
                                );
                            }
                        } else {
                            context.insert(
                                key.to_owned(),
                                Define {
                                    args: None,
                                    statement: args[i].to_owned(),
                                    call: false,
                                },
                            );
                        }
                    }
                }
            }
        }
        let mut iter = d.statement.clone().into_iter().peekable();
        while let Some(token) = iter.next() {
            match &token.token() {
                Token::Directive => {
                    if let Some(tp) = iter.peek() {
                        match tp.token() {
                            Token::Word(_) => {
                                if let Token::Word(w) = iter.next().unwrap().token() {
                                    ret.push(TokenPos::with_pos(Token::DoubleQuote, &token));
                                    ret.append(&mut _resolve_word(
                                        &mut iter,
                                        &w,
                                        &token,
                                        resolver,
                                        &mut context,
                                    ));
                                    ret.push(TokenPos::with_pos(Token::DoubleQuote, &token));
                                }
                            }
                            Token::Directive => {
                                iter.next();
                            }
                            _ => {}
                        }
                    }
                }
                Token::Word(w) => {
                    ret.append(&mut _resolve_word(
                        &mut iter,
                        w,
                        &token,
                        resolver,
                        &mut context,
                    ));
                }
                _ => ret.push(token.to_owned()),
            }
        }
        Some(ret)
    } else {
        None
    }
}

fn _resolve_word<R>(
    iter: &mut Peekable<IntoIter<TokenPos>>,
    ident: &str,
    token: &TokenPos,
    resolver: R,
    mut defines: &mut HashMap<String, Define>,
) -> Vec<TokenPos>
where
    R: Fn(&str) -> String + Copy,
{
    if let Some(d2) = defines.get(ident) {
        if d2.call {
            if let Some(r) = _resolve(
                ident,
                &Define {
                    call: false,
                    args: Some(
                        read_args!(iter)
                            .into_iter()
                            .map(|arg| _preprocess(arg, resolver, &mut defines))
                            .collect::<Result<Vec<Vec<TokenPos>>, String>>()
                            .unwrap(),
                    ),
                    statement: Vec::new(),
                },
                resolver,
                &defines,
            ) {
                return r;
            }
        } else if let Some(r) = _resolve(ident, d2, resolver, &defines) {
            return r;
        } else {
            return vec![token.to_owned()];
        }
    }
    vec![token.to_owned()]
}

pub fn preprocess<R>(source: Vec<TokenPos>, resolver: R) -> Result<Vec<TokenPos>, String>
where
    R: Fn(&str) -> String + Copy,
{
    let mut defines: HashMap<String, Define> = HashMap::new();
    _preprocess(source, resolver, &mut defines)
}

pub fn _preprocess<R>(
    source: Vec<TokenPos>,
    resolver: R,
    mut defines: &mut std::collections::HashMap<std::string::String, define::Define>,
) -> Result<Vec<TokenPos>, String>
where
    R: Fn(&str) -> String + Copy,
{
    let mut ret = Vec::new();
    let mut iter = source.into_iter().peekable();
    let mut if_state = IfStates::new();
    while let Some(token) = iter.next() {
        match (&token.token(), if_state.reading()) {
            (Token::Directive, r) => {
                if let Token::Word(directive) = iter.next().unwrap().token() {
                    match (directive.as_str(), r) {
                        ("define", true) => {
                            skip_whitespace!(iter);
                            if let Some(tp) = iter.next() {
                                if let Token::Word(name) = tp.token() {
                                    // skip_whitespace!(iter);
                                    let args = if let Some(tp) = iter.peek() {
                                        if tp.token() == &Token::LeftParenthesis {
                                            let args = read_args!(iter)
                                                .into_iter()
                                                .map(|arg| _preprocess(arg, resolver, &mut defines))
                                                .collect::<Result<Vec<Vec<TokenPos>>, String>>()
                                                .unwrap();
                                            Some(args)
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    };
                                    let body = read_line!(iter);
                                    defines.insert(
                                        name.to_owned(),
                                        Define {
                                            call: args.is_some(),
                                            args,
                                            statement: body,
                                        },
                                    );
                                } else {
                                    return Err("define without name".to_string());
                                }
                            }
                        }
                        ("undef", true) => {
                            skip_whitespace!(iter);
                            if let Some(tp) = iter.next() {
                                if let Token::Word(name) = tp.token().clone() {
                                    defines.remove(&name);
                                } else {
                                    return Err("undef without name".to_string());
                                }
                            } else {
                                return Err("undef without name".to_string());
                            }
                        }
                        ("ifdef", true) => {
                            skip_whitespace!(iter);
                            if let Some(tp) = iter.next() {
                                if let Token::Word(name) = tp.token().clone() {
                                    if defines.contains_key(&name) {
                                        if_state.push(IfState::ReadingIf);
                                    } else {
                                        if_state.push(IfState::PassingIf);
                                    }
                                }
                            }
                        }
                        ("ifndef", true) => {
                            skip_whitespace!(iter);
                            if let Some(tp) = iter.next() {
                                if let Token::Word(name) = tp.token().clone() {
                                    if defines.contains_key(&name) {
                                        if_state.push(IfState::PassingIf);
                                    } else {
                                        if_state.push(IfState::ReadingIf);
                                    }
                                }
                            }
                        }
                        ("ifdef", false) => {
                            if_state.push(IfState::PassingChild);
                        }
                        ("ifndef", false) => {
                            if_state.push(IfState::PassingChild);
                        }
                        ("else", _) => if_state.flip(),
                        ("endif", _) => {
                            if_state.pop();
                        }
                        ("include", true) => {
                            let file = render(read_line!(iter))
                                .export()
                                .trim_matches('"')
                                .to_owned();
                            ret.append(&mut _preprocess(
                                super::tokenize(&resolver(&file), &file).unwrap(),
                                resolver,
                                defines,
                            )?);
                        }
                        ("include", _) => {
                            read_line!(iter);
                        }
                        _ => {
                            error!("Unknown directive: {:?}", directive);
                            read_line!(iter);
                        }
                    }
                }
            }
            (Token::Word(text), true) => {
                if defines.contains_key(text) {
                    ret.append(
                        &mut _resolve(
                            &text,
                            &Define {
                                call: false,
                                args: if let Some(tp) = iter.peek() {
                                    if tp.token() == &Token::LeftParenthesis {
                                        Some(
                                            read_args!(iter)
                                                .into_iter()
                                                .map(|arg| _preprocess(arg, resolver, &mut defines))
                                                .collect::<Result<Vec<Vec<TokenPos>>, String>>()
                                                .unwrap(),
                                        )
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                },
                                statement: Vec::new(),
                            },
                            resolver,
                            &defines,
                        )
                        .unwrap(),
                    );
                } else {
                    ret.push(token);
                }
            }
            (_, true) => {
                ret.push(token);
            }
            _ => {}
        }
    }
    Ok(ret)
}
