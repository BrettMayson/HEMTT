use std::iter::Peekable;
use std::vec::IntoIter;
use std::{collections::HashMap, fs::read_to_string, path::PathBuf};

use pest::error::Error;
use pest::Parser;

mod token;
#[cfg(test)]
use token::Whitespace;
use token::{PreProcessParser, Rule, Token};

mod render;
pub use render::render;

mod define;
use define::Define;
mod ifstate;
use ifstate::{IfState, IfStates};

pub fn tokenize(source: &str) -> Result<Vec<Token>, Error<Rule>> {
    let mut tokens = Vec::new();

    let pairs = PreProcessParser::parse(Rule::file, source)?;
    for pair in pairs {
        tokens.push(Token::from(pair))
    }

    Ok(tokens)
}

macro_rules! skip_whitespace {
    ($i: ident) => {{
        let mut next = $i.peek();
        while let Some(Token::Whitespace(_)) = next {
            $i.next();
            next = $i.peek();
        }
    }};
}

macro_rules! read_args {
    ($i: ident) => {{
        let mut ret: Vec<Vec<Token>> = Vec::new();
        let mut next = $i.next();
        let mut arg: Vec<Token> = Vec::new();
        let mut level = 0;
        if let Some(Token::LeftParenthesis) = next {
            next = $i.next();
        }
        loop {
            match next {
                Some(Token::LeftParenthesis) => {
                    level += 1;
                    arg.push(Token::LeftParenthesis);
                }
                Some(Token::RightParenthesis) => {
                    if level == 0 {
                        if !arg.is_empty() {
                            ret.push(arg);
                        }
                        break;
                    } else {
                        arg.push(Token::RightParenthesis);
                    }
                    level -= 1;
                }
                Some(Token::Comma) => {
                    if level == 0 {
                        if !arg.is_empty() {
                            ret.push(arg);
                            arg = Vec::new();
                        }
                    } else {
                        arg.push(Token::Comma);
                    }
                }
                Some(t) => arg.push(t),
                None => break,
            }
            next = $i.next();
        }
        ret
    }};
}

#[test]
fn test_read_args() {
    let tokens = tokenize("(B(C); call f);").unwrap();
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
        read_args!(a)
    )
}

macro_rules! read_line {
    ($i: ident) => {{
        let mut ret: Vec<Token> = Vec::new();
        let mut next = $i.next();
        // Skip initial whitespace
        while let Some(Token::Whitespace(_)) = next {
            next = $i.next();
        }
        loop {
            match next {
                Some(Token::Newline) => break,
                Some(Token::Escape) => {
                    if $i.peek() == Some(&Token::Newline) {
                        $i.next();
                    }
                    next = $i.next();
                    while let Some(Token::Whitespace(_)) = next {
                        next = $i.next();
                    }
                }
                Some(n) => {
                    ret.push(n);
                    next = $i.next();
                }
                _ => break,
            }
        }
        ret
    }};
}

#[test]
fn test_read_line() {
    let tokens = tokenize("test = false;\ntest = true;\n").unwrap();
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
    )
}

pub fn _resolve(
    ident: &str,
    define: &Define,
    resolver: &dyn Fn(&str) -> PathBuf,
    defines: &HashMap<String, Define>,
) -> Option<Vec<Token>> {
    if let Some(d) = defines.get(ident) {
        let mut ret = Vec::new();
        let mut context = defines.to_owned();
        if let Some(dargs) = &d.args {
            if let Some(args) = &define.args {
                if dargs.len() != args.len() {
                    panic!("Invalid arg lengths");
                }
                for i in 0..dargs.len() {
                    if let Token::Word(key) = &dargs[i][0] {
                        if args[i].len() == 1 {
                            if let Token::Word(value) = &args[i][0] {
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
            match &token {
                Token::Directive => match iter.peek() {
                    Some(Token::Word(_)) => {
                        if let Token::Word(w) = iter.next().unwrap() {
                            ret.push(Token::DoubleQuote);
                            ret.append(&mut _resolve_word(
                                &mut iter,
                                &w,
                                &token,
                                resolver,
                                &mut context,
                            ));
                            ret.push(Token::DoubleQuote);
                        }
                    }
                    Some(Token::Directive) => {
                        iter.next();
                    }
                    _ => {}
                },
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

fn _resolve_word(
    iter: &mut Peekable<IntoIter<Token>>,
    ident: &str,
    token: &Token,
    resolver: &dyn Fn(&str) -> PathBuf,
    mut defines: &mut HashMap<String, Define>,
) -> Vec<Token> {
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
                            .collect::<Result<Vec<Vec<Token>>, String>>()
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

pub fn preprocess(
    source: Vec<Token>,
    resolver: &dyn Fn(&str) -> PathBuf,
) -> Result<Vec<Token>, String> {
    let mut defines: HashMap<String, Define> = HashMap::new();
    _preprocess(source, resolver, &mut defines)
}

pub fn _preprocess(
    source: Vec<Token>,
    resolver: &dyn Fn(&str) -> PathBuf,
    mut defines: &mut std::collections::HashMap<std::string::String, define::Define>,
) -> Result<Vec<Token>, String> {
    let mut ret = Vec::new();
    let mut iter = source.into_iter().peekable();
    let mut if_state = IfStates::new();
    while let Some(token) = iter.next() {
        match (&token, if_state.reading()) {
            (Token::Directive, r) => {
                if let Token::Word(directive) = iter.next().unwrap() {
                    match (directive.as_str(), r) {
                        ("define", true) => {
                            skip_whitespace!(iter);
                            if let Some(Token::Word(name)) = iter.next() {
                                // skip_whitespace!(iter);
                                let args = if iter.peek() == Some(&Token::LeftParenthesis) {
                                    let args = read_args!(iter)
                                        .into_iter()
                                        .map(|arg| _preprocess(arg, resolver, &mut defines))
                                        .collect::<Result<Vec<Vec<Token>>, String>>()
                                        .unwrap();
                                    Some(args)
                                } else {
                                    None
                                };
                                let body = read_line!(iter);
                                defines.insert(
                                    name,
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
                        ("undef", true) => {
                            skip_whitespace!(iter);
                            if let Some(Token::Word(name)) = iter.next() {
                                defines.remove(&name);
                            } else {
                                return Err("undef without name".to_string());
                            }
                        }
                        ("ifdef", true) => {
                            skip_whitespace!(iter);
                            if let Some(Token::Word(name)) = iter.next() {
                                if defines.contains_key(&name) {
                                    if_state.push(IfState::ReadingIf);
                                } else {
                                    if_state.push(IfState::PassingIf);
                                }
                            }
                        }
                        ("ifndef", true) => {
                            skip_whitespace!(iter);
                            if let Some(Token::Word(name)) = iter.next() {
                                if defines.contains_key(&name) {
                                    if_state.push(IfState::PassingIf);
                                } else {
                                    if_state.push(IfState::ReadingIf);
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
                            let file = render(read_line!(iter)).trim_matches('"').to_owned();
                            ret.append(&mut _preprocess(
                                super::tokenize(&read_to_string(resolver(&file)).unwrap()).unwrap(),
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
                                args: if iter.peek() == Some(&Token::LeftParenthesis) {
                                    Some(
                                        read_args!(iter)
                                            .into_iter()
                                            .map(|arg| _preprocess(arg, resolver, &mut defines))
                                            .collect::<Result<Vec<Vec<Token>>, String>>()
                                            .unwrap(),
                                    )
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
                    ret.push(Token::Word(text.to_owned()));
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
