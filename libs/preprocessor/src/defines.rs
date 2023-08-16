use std::collections::HashMap;

use strsim::levenshtein;

use crate::{definition::Definition, token::Token};

type InnerDefines = HashMap<String, (Token, Definition)>;

#[derive(Default)]
/// `HashMap` of all current defines
pub struct Defines {
    global: InnerDefines,
    stack: Vec<(String, InnerDefines)>,
}

impl Defines {
    pub fn contains_key(&self, key: &str) -> bool {
        println!(
            "check key {}, stack: {:?}, global: {:?}",
            key,
            self.stack.last().map(|i| i.1.keys()),
            self.global.keys()
        );
        // self.stack.last().map_or(false, |i| i.1.contains_key(key)) || self.global.contains_key(key)
        if let Some(last) = self.stack.last() {
            if last.0 == key {
                return false;
            }
            if last.1.contains_key(key) {
                return true;
            }
        }
        self.global.contains_key(key)
    }

    pub fn get(&self, key: &str) -> Option<&(Token, Definition)> {
        println!("get {}", key);
        self.stack
            .last()
            .as_ref()
            .and_then(|i| i.1.get(key))
            .or_else(|| {
                println!("get global {}", key);
                self.global.get(key)
            })
    }

    pub fn insert(&mut self, key: String, value: (Token, Definition)) {
        if let Some(stack) = self.stack.last_mut() {
            println!("insert {} into stack = {:?}", key, value.1);
            stack.1.insert(key, value);
        } else {
            self.global.insert(key, value);
        }
    }

    pub fn remove(&mut self, key: &str) {
        if let Some(scope) = self.stack.last_mut() {
            scope.1.remove(key);
        } else {
            self.global.remove(key);
        }
    }

    pub fn push(&mut self, name: String, args: InnerDefines) {
        println!("push {:?}", args.keys());
        self.stack.push((name, args));
    }

    pub fn pop(&mut self) {
        println!("pop");
        self.stack.pop();
    }

    pub fn stack(&self) -> &Vec<(String, InnerDefines)> {
        &self.stack
    }

    pub fn global(&self) -> &InnerDefines {
        &self.global
    }

    pub fn similar_function(&self, search: &str, args: Option<usize>) -> Vec<&str> {
        let mut similar = self
            .global
            .iter()
            .filter(|(_, (_, def))| {
                let Definition::Function(func) = def else {
                    return false;
                };
                args.map_or(true, |args| func.args().len() == args)
            })
            .map(|(name, _)| (name.as_str(), levenshtein(name, search)))
            .collect::<Vec<_>>();
        similar.sort_by_key(|(_, v)| *v);
        similar.retain(|s| s.1 <= 3);
        if similar.len() > 3 {
            similar.truncate(3);
        }
        similar.into_iter().map(|(n, _)| n).collect::<Vec<_>>()
    }

    pub fn similar_values(&self, search: &str) -> Vec<&str> {
        let mut similar = self
            .global
            .iter()
            .filter(|(_, (_, def))| {
                let Definition::Value(_) = def else {
                    return false;
                };
                true
            })
            .map(|(name, _)| (name.as_str(), levenshtein(name, search)))
            .collect::<Vec<_>>();
        similar.sort_by_key(|(_, v)| *v);
        similar.retain(|s| s.1 <= 3);
        if similar.len() > 3 {
            similar.truncate(3);
        }
        similar.into_iter().map(|(n, _)| n).collect::<Vec<_>>()
    }
}
