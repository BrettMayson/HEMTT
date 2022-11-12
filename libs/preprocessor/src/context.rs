use std::{
    collections::HashMap,
    sync::{atomic::AtomicUsize, Arc},
};

use hemtt_tokens::{symbol::Symbol, Token};

use crate::{ifstate::IfStates, Error};

const BUILTIN: [&str; 37] = [
    "__LINE__",
    "__FILE__",
    "__DATE_ARR__",
    "__DATE_STR__",
    "__DATE_STR_ISO8601__",
    "__TIME__",
    "__TIME_UTC__",
    "__TIMESTAMP_UTC__",
    "__COUNTER__",
    "__COUNTER_RESET__",
    "__RAND_INT8__",
    "__RAND_INT16__",
    "__RAND_INT32__",
    "__RAND_INT64__",
    "__RAND_UINT8__",
    "__RAND_UINT16__",
    "__RAND_UINT32__",
    "__RAND_UINT64__",
    "__ARMA__",
    "__ARMA3__",
    "__A3_DEBUG__",
    "__HEMTT__",
    "__HEMTT_DEBUG__",
    "__HEMTT_VERSION__",
    "__HEMTT_VERSION_MAJ__",
    "__HEMTT_VERSION_MIN__",
    "__HEMTT_VERSION_REV__",
    "__HEMTT_VERSION_BUILD__",
    "__HEMTT_PROJECT_NAME__",
    "__HEMTT_PROJECT_VERSION__",
    "__HEMTT_PROJECT_VERSION_MAJ__",
    "__HEMTT_PROJECT_VERSION_MIN__",
    "__HEMTT_PROJECT_VERSION_REV__",
    "__HEMTT_PROJECT_VERSION_BUILD__",
    "__HEMTT_PROJECT_MAINPREFIX__",
    "__HEMTT_PROJECT_PREFIX__",
    "__HEMTT_PROJECT_AUTHOR__",
];

#[derive(Clone, Debug)]
pub struct Context {
    ifstates: IfStates,
    definitions: HashMap<String, (Token, Definition)>,
    entry: String,
    current_file: String,
    counter: Arc<AtomicUsize>,
}

impl Context {
    #[must_use]
    pub fn new(entry: String) -> Self {
        Self {
            ifstates: IfStates::new(),
            definitions: HashMap::new(),
            current_file: entry.clone(),
            entry,
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    #[must_use]
    pub const fn ifstates(&self) -> &IfStates {
        &self.ifstates
    }

    pub fn ifstates_mut(&mut self) -> &mut IfStates {
        &mut self.ifstates
    }

    #[must_use]
    pub const fn definitions(&self) -> &HashMap<String, (Token, Definition)> {
        &self.definitions
    }

    pub fn definitions_mut(&mut self) -> &mut HashMap<String, (Token, Definition)> {
        &mut self.definitions
    }

    #[must_use]
    pub const fn entry(&self) -> &String {
        &self.entry
    }

    #[must_use]
    pub const fn current_file(&self) -> &String {
        &self.current_file
    }

    pub fn set_current_file(&mut self, file: String) {
        self.current_file = file;
    }

    /// Define a macro
    ///
    /// # Errors
    /// If the macro is a builtin macro
    pub fn define(
        &mut self,
        ident: String,
        source: Token,
        definition: Definition,
    ) -> Result<(), Error> {
        if BUILTIN.contains(&ident.as_str()) {
            return Err(Error::ChangeBuiltin {
                token: Box::new(source),
            });
        }
        self.definitions.insert(ident, (source, definition));
        Ok(())
    }

    /// Undefine a macro
    ///
    /// # Errors
    /// If the macro is a builtin macro
    pub fn undefine(
        &mut self,
        ident: &str,
        source: &Token,
    ) -> Result<Option<(Token, Definition)>, Error> {
        if BUILTIN.contains(&ident) {
            return Err(Error::ChangeBuiltin {
                token: Box::new(source.clone()),
            });
        }
        Ok(self.definitions.remove(ident))
    }

    #[must_use]
    pub fn has(&self, ident: &str) -> bool {
        self.definitions.contains_key(ident)
    }

    #[must_use]
    pub fn get(&self, ident: &str, token: &Token) -> Option<(Token, Definition)> {
        match ident {
            "__LINE__" => Some((
                Token::builtin(),
                Definition::Value(vec![Token::new(
                    Symbol::Word(token.source().start().1 .0.to_string()),
                    token.source().clone(),
                )]),
            )),
            "__FILE__" => Some((
                Token::builtin(),
                Definition::Value(vec![Token::new(
                    Symbol::Word(token.source().path().to_string().replace('\\', "/")),
                    token.source().clone(),
                )]),
            )),
            "__COUNTER__" => Some((
                Token::builtin(),
                Definition::Value(vec![Token::new(
                    Symbol::Word(
                        self.counter
                            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                            .to_string(),
                    ),
                    token.source().clone(),
                )]),
            )),
            "__COUNTER_RESET__" => {
                self.counter.store(0, std::sync::atomic::Ordering::SeqCst);
                Some((
                    Token::builtin(),
                    Definition::Value(vec![Token::new(Symbol::Void, token.source().clone())]),
                ))
            }
            "__ARMA__" | "__ARMA3__" | "__HEMTT__" => Some((
                Token::builtin(),
                Definition::Value(vec![Token::new(Symbol::Digit(1), token.source().clone())]),
            )),
            _ => self
                .definitions
                .get(ident)
                .map(|(source, definition)| (source.clone(), definition.clone())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Definition {
    Function(FunctionDefinition),
    Value(Vec<Token>),
    Unit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionDefinition {
    parameters: Vec<Token>,
    body: Vec<Token>,
}

impl FunctionDefinition {
    #[must_use]
    pub fn new(parameters: Vec<Token>, body: Vec<Token>) -> Self {
        Self { parameters, body }
    }

    #[must_use]
    pub fn parameters(&self) -> &[Token] {
        &self.parameters
    }

    #[must_use]
    pub fn body(&self) -> &[Token] {
        &self.body
    }
}
