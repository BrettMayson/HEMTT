use std::{collections::HashMap, rc::Rc};

use hemtt_common::position::Position;
use strsim::levenshtein;

use crate::{definition::Definition, symbol::Symbol, token::Token};

type InnerDefines = HashMap<Rc<str>, (Token, Definition)>;

#[derive(Default)]
/// `HashMap` of all current defines
pub struct Defines {
    global: InnerDefines,
    stack: Vec<(Rc<str>, InnerDefines)>,

    counter: u16,
}

const BUILTIN: [&str; 4] = ["__COUNTER__", "__COUNTER_RESET__", "__FILE__", "__LINE__"];

impl Defines {
    pub fn contains_key(&self, key: &str) -> bool {
        if BUILTIN.contains(&key) {
            return true;
        }
        if let Some(last) = self.stack.last() {
            if *last.0 == *key {
                return false;
            }
            if last.1.contains_key(key) {
                return true;
            }
        }
        self.global.contains_key(key)
    }

    pub fn get(&mut self, key: &Token, site: &Position) -> Option<(Token, Definition)> {
        let ident = key.to_string();
        if BUILTIN.contains(&ident.as_str()) {
            match ident.as_str() {
                "__COUNTER__" => {
                    let counter = self.counter;
                    self.counter += 1;
                    return Some((
                        Token::new(Symbol::Directive, Position::builtin()),
                        Definition::Value(vec![Token::new(
                            Symbol::Digit(counter.into()),
                            key.source().clone(),
                        )]),
                    ));
                }
                "__COUNTER_RESET__" => {
                    self.counter = 0;
                    return Some((
                        Token::new(Symbol::Directive, Position::builtin()),
                        Definition::Void,
                    ));
                }
                "__FILE__" => {
                    return Some((
                        Token::new(Symbol::Directive, Position::builtin()),
                        Definition::Value(vec![Token::new(
                            Symbol::Word(site.path_or_builtin()),
                            key.source().clone(),
                        )]),
                    ));
                }
                "__LINE__" => {
                    return Some((
                        Token::new(Symbol::Directive, Position::builtin()),
                        Definition::Value(vec![Token::new(
                            Symbol::Digit(site.start().1 .0),
                            key.source().clone(),
                        )]),
                    ));
                }
                _ => unreachable!(),
            }
        }
        if let Some(last) = self.stack.last() {
            if *last.0 == *ident {
                return None;
            }
        }
        self.stack
            .last()
            .as_ref()
            .and_then(|i| i.1.get(ident.as_str()))
            .or_else(|| self.global.get(ident.as_str()))
            .cloned()
    }

    #[cfg(test)]
    pub fn get_test(&self, key: &str) -> Option<&(Token, Definition)> {
        self.stack
            .last()
            .as_ref()
            .and_then(|i| i.1.get(key))
            .or_else(|| self.global.get(key))
    }

    pub fn insert(&mut self, key: &str, value: (Token, Definition)) {
        if let Some(stack) = self.stack.last_mut() {
            stack.1.insert(Rc::from(key), value);
        } else {
            self.global.insert(Rc::from(key), value);
        }
    }

    pub fn remove(&mut self, key: &str) {
        if let Some(scope) = self.stack.last_mut() {
            scope.1.remove(key);
        } else {
            self.global.remove(key);
        }
    }

    pub fn push(&mut self, name: &str, args: InnerDefines) {
        self.stack.push((Rc::from(name), args));
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub const fn stack(&self) -> &Vec<(Rc<str>, InnerDefines)> {
        &self.stack
    }

    pub const fn global(&self) -> &InnerDefines {
        &self.global
    }

    pub fn similar_function(&self, search: &str, args: Option<usize>) -> Vec<&Rc<str>> {
        let mut similar = self
            .global
            .iter()
            .filter(|(_, (_, def))| {
                let Definition::Function(func) = def else {
                    return false;
                };
                args.map_or(true, |args| func.args().len() == args)
            })
            .map(|(name, _)| (name, levenshtein(name, search)))
            .collect::<Vec<_>>();
        similar.sort_by_key(|(_, v)| *v);
        similar.retain(|s| s.1 <= 3);
        if similar.len() > 3 {
            similar.truncate(3);
        }
        similar.into_iter().map(|(n, _)| n).collect::<Vec<_>>()
    }

    pub fn similar_values(&self, search: &str) -> Vec<&Rc<str>> {
        let mut similar = self
            .global
            .iter()
            .filter(|(_, (_, def))| {
                let Definition::Value(_) = def else {
                    return false;
                };
                true
            })
            .map(|(name, _)| (name, levenshtein(name, search)))
            .collect::<Vec<_>>();
        similar.sort_by_key(|(_, v)| *v);
        similar.retain(|s| s.1 <= 3);
        if similar.len() > 3 {
            similar.truncate(3);
        }
        similar.into_iter().map(|(n, _)| n).collect::<Vec<_>>()
    }
}
