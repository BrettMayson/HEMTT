use std::{
    collections::HashMap,
    sync::{atomic::AtomicUsize, Arc},
};

use hemtt_error::tokens::{Symbol, Token};
use vfs::VfsPath;

use crate::{defines::Defines, ifstate::IfStates, Error};

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
/// Preprocessor context
pub struct Context<'a> {
    ifstates: IfStates,
    definitions: Defines,
    entry: VfsPath,
    current_file: VfsPath,
    counter: Arc<AtomicUsize>,
    trace: Vec<Token>,
    parent: Option<&'a Self>,
}

impl<'a> Context<'a> {
    #[must_use]
    /// Create a new `Context`
    pub fn new(entry: VfsPath) -> Self {
        Self {
            ifstates: IfStates::new(),
            definitions: HashMap::new(),
            current_file: entry.clone(),
            entry,
            counter: Arc::new(AtomicUsize::new(0)),
            trace: Vec::new(),
            parent: None,
        }
    }

    #[must_use]
    /// Create a new `Context` from a parent
    pub fn stack(&'a self, source: Token) -> Context<'a> {
        Self {
            ifstates: self.ifstates.clone(),
            definitions: HashMap::new(),
            current_file: self.current_file.clone(),
            entry: self.entry.clone(),
            counter: self.counter.clone(),
            trace: {
                let mut trace = self.trace.clone();
                trace.push(source);
                trace
            },
            parent: Some(self),
        }
    }

    /// Push a [`Token`] to the trace
    pub fn push(&mut self, source: Token) {
        self.trace.push(source);
    }

    /// Pop a [`Token`] from the trace
    pub fn pop(&mut self) -> Option<Token> {
        self.trace.pop()
    }

    #[must_use]
    /// Get the current trace
    pub fn trace(&self) -> Vec<Token> {
        self.trace.clone()
    }

    #[must_use]
    /// Get the current [`IfState`](crate::ifstate::IfState)
    pub const fn ifstates(&self) -> &IfStates {
        &self.ifstates
    }

    /// Get the current [`IfState`](crate::ifstate::IfState) mutably
    pub fn ifstates_mut(&mut self) -> &mut IfStates {
        &mut self.ifstates
    }

    #[must_use]
    /// Get the current [`Definition`]s
    pub const fn definitions(&self) -> &Defines {
        &self.definitions
    }

    /// Get the current [`Definition`]s mutably
    pub fn definitions_mut(&mut self) -> &mut Defines {
        &mut self.definitions
    }

    #[must_use]
    /// Get the entry name
    pub const fn entry(&self) -> &VfsPath {
        &self.entry
    }

    #[must_use]
    /// Get the current file
    pub const fn current_file(&self) -> &VfsPath {
        &self.current_file
    }

    /// Set the current file
    pub fn set_current_file(&mut self, file: VfsPath) {
        self.current_file = file;
    }

    /// Define a macro [`Definition`]
    ///
    /// # Errors
    /// [`Error::ChangeBuiltin`] if the macro is a builtin macro
    pub fn define(
        &mut self,
        ident: String,
        source: Token,
        definition: Definition,
    ) -> Result<(), Error> {
        if BUILTIN.contains(&ident.as_str()) {
            return Err(Error::ChangeBuiltin {
                token: Box::new(source),
                trace: self.trace(),
            });
        }
        self.definitions.insert(ident, (source, definition));
        Ok(())
    }

    /// Undefine a macro [`Definition`]
    ///
    /// # Errors
    /// [`Error::ChangeBuiltin`] if the macro is a builtin macro
    pub fn undefine(
        &mut self,
        ident: &str,
        source: &Token,
    ) -> Result<Option<(Token, Definition)>, Error> {
        if BUILTIN.contains(&ident) {
            return Err(Error::ChangeBuiltin {
                token: Box::new(source.clone()),
                trace: self.trace(),
            });
        }
        Ok(self.definitions.remove(ident))
    }

    #[must_use]
    /// Check if a macro [`Definition`] exists
    pub fn has(&self, ident: &str) -> bool {
        self.definitions.contains_key(ident)
    }

    #[must_use]
    /// Get a macro [`Definition`]
    pub fn get(&self, ident: &str, token: &Token) -> Option<(Token, Definition)> {
        match ident {
            "__LINE__" => Some((
                Token::builtin(Some(Box::new(token.clone()))),
                Definition::Value(vec![Token::new(
                    Symbol::Word(token.source().start().1 .0.to_string()),
                    token.source().clone(),
                    Some(Box::new(token.clone())),
                )]),
            )),
            "__FILE__" => Some((
                Token::builtin(Some(Box::new(token.clone()))),
                Definition::Value(vec![Token::new(
                    Symbol::Word(token.source().path().map_or_else(
                        || String::from("%builtin%"),
                        |p| p.as_str().replace('\\', "/"),
                    )),
                    token.source().clone(),
                    Some(Box::new(token.clone())),
                )]),
            )),
            "__COUNTER__" => Some((
                Token::builtin(Some(Box::new(token.clone()))),
                Definition::Value(vec![Token::new(
                    Symbol::Word(
                        self.counter
                            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                            .to_string(),
                    ),
                    token.source().clone(),
                    Some(Box::new(token.clone())),
                )]),
            )),
            "__COUNTER_RESET__" => {
                self.counter.store(0, std::sync::atomic::Ordering::SeqCst);
                Some((
                    Token::builtin(Some(Box::new(token.clone()))),
                    Definition::Value(vec![Token::new(
                        Symbol::Void,
                        token.source().clone(),
                        Some(Box::new(token.clone())),
                    )]),
                ))
            }
            "__ARMA__" | "__ARMA3__" | "__HEMTT__" => Some((
                Token::builtin(Some(Box::new(token.clone()))),
                Definition::Value(vec![Token::new(
                    Symbol::Digit(1),
                    token.source().clone(),
                    Some(Box::new(token.clone())),
                )]),
            )),
            _ => {
                // get locally or from parent
                let mut context = self;
                loop {
                    if let Some((source, definition)) = context.definitions.get(ident) {
                        return Some((source.clone(), definition.clone()));
                    }
                    if let Some(parent) = &context.parent {
                        context = parent;
                    } else {
                        break;
                    }
                }
                None
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// A macro definition
pub enum Definition {
    /// A [`FunctionDefinition`] that takes parameters
    Function(FunctionDefinition),
    /// A value that is a list of [`Token`]s to be added at the call site
    Value(Vec<Token>),
    /// A flag that can be checked with `#ifdef`
    /// Tokens are only used for error reporting
    Unit(Vec<Token>),
}

impl Definition {
    #[must_use]
    /// Check if the definition is a [`FunctionDefinition`]
    pub const fn is_function(&self) -> bool {
        matches!(self, Self::Function(_))
    }

    #[must_use]
    /// Check if the definition is a value
    pub const fn is_value(&self) -> bool {
        matches!(self, Self::Value(_))
    }

    #[must_use]
    /// Check if the definition is a flag
    pub const fn is_unit(&self) -> bool {
        matches!(self, Self::Unit(_))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// A function definition
///
/// # Examples
///
/// ```cpp
/// #define QUOTE(x) #x
/// #define FOO(a, b) QUOTE(a + b)
/// my_value = FOO(1, 2);
/// ```
pub struct FunctionDefinition {
    parameters: Vec<Token>,
    body: Vec<Token>,
}

impl FunctionDefinition {
    #[must_use]
    /// Create a new [`FunctionDefinition`]
    pub fn new(parameters: Vec<Token>, body: Vec<Token>) -> Self {
        Self { parameters, body }
    }

    #[must_use]
    /// Get the parameter [`Token`]s
    pub fn parameters(&self) -> &[Token] {
        &self.parameters
    }

    #[must_use]
    /// Get the body [`Token`]s
    pub fn body(&self) -> &[Token] {
        &self.body
    }
}
