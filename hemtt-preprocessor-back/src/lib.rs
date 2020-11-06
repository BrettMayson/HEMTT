#[macro_use]
extern crate log;

use lazy_static::lazy_static;

use regex::Regex;

mod define;
use define::Define;
mod state;
use state::State;

const ESCAPE_NEWLINE: &str = "$%\\%$";

#[derive(Debug)]
pub struct Preprocessor<'a> {
    pub source: &'a str,
    pub output: String,
}

impl<'a> Preprocessor<'a> {
    pub fn from_source(source: &'a str) -> Self {
        let mut output: Vec<String> = Vec::new();
        let _source = source.replace('\r', "");
        // This is sketch but it's good for right now
        let _source = _source.replace(" \\\n", ESCAPE_NEWLINE).replace("\\\n", ESCAPE_NEWLINE);
        let source_split: Vec<&str> = _source.split('\n').collect();

        let mut index = 0usize;

        let mut state = State::new();

        println!("split: {:?}", source_split);

        while index < source_split.len() {
            let line = source_split[index].replace(ESCAPE_NEWLINE, "\n");
            trace!("{} > {}", index, line);
            if line.starts_with('#') {
                if let Some(s) = Self::directive(&mut state, &line) {
                    output.push(s);
                }
            } else if let Ok(s) = Self::process_line(&mut state, &line) {
                output.push(s);
            }

            index += 1;
        }
        
        Self {
            source,
            output: output.join("\n"),
        }
    }

    pub fn directive(state: &mut State, line: &str) -> Option<String> {
        println!("Directive: {}", line);
        let sections = line.split(' ').collect::<Vec<&str>>();
        match sections.first() {
            Some(&"#include") => state.include(sections),
            Some(&"#define") => state.define(sections[1..].join(" ")),
            Some(&"#undef") => state.undefine(sections),
            Some(&"#if") => state._if(sections),
            Some(&"#else") => state._else(sections),
            Some(&"#end") => state._end(sections),
            Some(&&_) => {
                if let Ok(s) = Self::process_line(state, line) {
                    return Some(s);
                }
                Ok(())
            },
            None => Ok(())
        }.unwrap();
        None
    }

    pub fn process_line(state: &mut State, line: &str) -> Result<String, ()> {
        let original = line.to_owned();
        let mut line = line.to_owned();
        line = Self::check_call(state, &line)?;
        println!("> {}", line);
        line = Self::check_words(state, &line)?;
        if original != line {
            println!("Line different:\n```\n{}\n```\n```\n{}\n```\n", original, line);
            Self::process_line(state, &line)
        } else {
            Ok(line)
        }
    }

    fn check_call(state: &mut State, line: &str) -> Result<String, ()> {
        lazy_static! {
            static ref RE_CALL: Regex = Regex::new(r#"(?s)(\w+)\(((?:"(?:[^"\\]|\\.)*"|\((?:[^)])*\)|.)*?)\)"#).unwrap();
        }
        let mut ret = line.to_owned();
        let original = line.to_owned();
        'outer: for cap in RE_CALL.captures_iter(&original) {
            if let Some(define) = state.get(&cap[1]) {
                println!("MacroCall Ident: {} Args: {}", &cap[1], &cap[2]);
                let inner = Self::check_call(state, &cap[2])?;
                if inner != cap[2] {
                    ret = Self::check_call(state, &format!(
                        "{}{}{}",
                        &line[..cap.get(2).unwrap().start()],
                        inner,
                        &line[cap.get(2).unwrap().end()..]
                    ))?;
                } else if let Some(define_args) = &define.args {
                    lazy_static! {
                        // static ref RE_ARGS: Regex = Regex::new(r#"([^",]+|"(?:[^"\\]|\\.)*")"#).unwrap();
                        static ref RE_ARGS: Regex = Regex::new(r#"([^,]+|"(?:[^\\]|\\.)*")"#).unwrap();
                    }
                    let call = cap[2].to_string();
                    println!("Checking args: {:?} = {}", define_args, call);
                    if define_args.len() > 1 {
                        let args: Vec<regex::Captures> = RE_ARGS.captures_iter(&call).collect();
                        if args.len() == define_args.len() {
                            let mut call_state = state.clone();
                            for i in 0..define_args.len() {
                                let value = args[i].get(1).unwrap().as_str().trim_matches(|c| c == '\n' || c == ' ');
                                let value = if let Some(existing) = state.get(value) {
                                    existing.statement
                                } else {
                                    value.to_string()
                                };
                                println!("tmp var: {} = {}", define_args[i], value);
                                call_state.insert(define_args[i].clone(), Define {
                                    ident: define_args[i].clone(),
                                    call: false,
                                    args: None,
                                    statement: value,
                                })?;
                            }
                            ret = Self::check_words(&mut call_state, &format!(
                                "{}{}{}",
                                &line[..cap.get(0).unwrap().start()],
                                define.statement,
                                &line[cap.get(0).unwrap().end()..]
                            ))?;
                            println!("macro call set line to: {}", ret);
                        } else {
                            panic!("Arg count mismatch calling {}", &cap[1]);
                        }
                    } else {
                        let mut call_state = state.clone();
                        let ident = define_args.get(0).unwrap().to_owned();
                        call_state.insert(ident.clone(), Define {
                            ident,
                            call: false,
                            args: None,
                            statement: call.trim_matches(|c| c == '\n' || c == ' ').to_string(),
                        })?;
                        ret = Self::check_words(&mut call_state, &format!(
                            "{}{}{}",
                            &line[..cap.get(0).unwrap().start()],
                            define.statement,
                            &line[cap.get(0).unwrap().end()..]
                        ))?;
                        println!("macro call set line to: {}", ret);
                    }
                }
                ret = Self::check_call(state, &ret)?;
                break 'outer;
            }
        }
        Ok(ret)
    }

    fn check_words(state: &mut State, line: &str) -> Result<String, ()> {
        lazy_static! {
            static ref RE_WORD: Regex = Regex::new(r"(\#?\#?\w+)").unwrap();
        }
        for cap in RE_WORD.captures_iter(&line.to_owned()) {
            let define = state.get(&cap[1].replace('#', ""));
            if cap[1].starts_with("##") && define.is_none() {
                return Self::process_line(state, &format!(
                    "{}{}{}",
                    &line[..cap.get(0).unwrap().start()],
                    &cap[1].replace("##",""),
                    &line[cap.get(0).unwrap().end()..]
                ));
            }
            if let Some(define) = define {
                if !define.call {
                    println!("word usage: `{}`", &cap[1]);
                    trace!("state: {:?}", state.defines());
                    if define.call {
                        continue;
                    }
                    let insert = if cap[1].starts_with("##") {
                        define.statement
                    } else if cap[1].starts_with('#') {
                        println!("quote: {}", define.statement);
                        format!("\"{}\"", define.statement)
                    } else {
                        define.statement
                    };
                    return Self::process_line(state, &format!(
                        "{}{}{}",
                        &line[..cap.get(0).unwrap().start()],
                        insert,
                        &line[cap.get(0).unwrap().end()..]
                    ));
                }
            }
        }
        Ok(line.to_string())
    }
}
