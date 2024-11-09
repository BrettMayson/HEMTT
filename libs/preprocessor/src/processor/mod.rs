use std::collections::HashMap;
use std::sync::Arc;

use hemtt_workspace::{
    position::Position,
    reporting::{Codes, Output, Processed, Symbol, Token},
    WorkspacePath,
};
use peekmore::{PeekMore, PeekMoreIterator};

use crate::codes::{pe26_unsupported_builtin::BuiltInNotSupported, pe2_unexpected_eof::UnexpectedEOF};
use crate::codes::pe3_expected_ident::ExpectedIdent;
use crate::codes::pw2_invalid_config_case::InvalidConfigCase;
use crate::codes::{pe18_eoi_ifstate::EoiIfState, pe25_exec::ExecNotSupported};
use crate::defines::Defines;
use crate::ifstate::IfStates;
use crate::Error;

use self::pragma::Pragma;

mod defines;
mod directives;
pub mod pragma;
mod whitespace;

#[derive(Default)]
/// Arma 3 Preprocessor
pub struct Processor {
    ifstates: IfStates,
    defines: Defines,

    included_files: Vec<WorkspacePath>,
    file_stack: Vec<WorkspacePath>,

    pub(crate) token_count: usize,

    macros: HashMap<String, Vec<Position>>,

    #[cfg(feature = "lsp")]
    /// Map of token usage to definition
    /// (token, definition)
    pub(crate) declarations: HashMap<Position, Position>,

    #[cfg(feature = "lsp")]
    /// Map of token definition to usage
    /// (definition, usages)
    pub(crate) usage: HashMap<Position, Vec<Position>>,

    /// Warnings
    pub(crate) warnings: Codes,

    /// The preprocessor was able to run checks, but the output should not be rapified
    pub(crate) no_rapify: bool,
}

impl Processor {
    #[must_use]
    /// Returns the defines
    pub const fn defines(&self) -> &Defines {
        &self.defines
    }

    /// Preprocess a file
    ///
    /// # Errors
    /// See [`Error`]
    pub fn run(path: &WorkspacePath) -> Result<Processed, (Vec<WorkspacePath>, Error)> {
        let mut processor = Self::default();

        processor.file_stack.push(path.clone());

        let tokens =
            crate::parse::parse(path).map_err(|e| (processor.included_files.clone(), e))?;
        let mut pragma = Pragma::root();
        let mut buffer = Vec::with_capacity(tokens.len());
        let mut stream = tokens.into_iter().peekmore();

        processor
            .file(&mut pragma, &mut stream, &mut buffer)
            .map_err(|e| (processor.included_files.clone(), e))?;

        if let Some(state) = processor.ifstates.pop() {
            return Err((
                processor.included_files,
                EoiIfState::code(state.token().as_ref().clone()),
            ));
        }

        if path.filename() == "Config.cpp" {
            processor
                .warnings
                .push(Arc::new(InvalidConfigCase::new(path.clone())));
        }

        Processed::new(
            buffer,
            processor.macros,
            #[cfg(feature = "lsp")]
            processor.usage,
            processor.warnings,
            processor.no_rapify,
        )
        .map_err(|e| (processor.included_files, e.into()))
    }

    fn file(
        &mut self,
        pragma: &mut Pragma,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
        buffer: &mut Vec<Output>,
    ) -> Result<(), Error> {
        loop {
            let first = stream.peek();
            if first.is_none() || first.expect("just checked").symbol().is_eoi() {
                return Ok(());
            }
            self.line(pragma, stream, buffer)?;
        }
    }

    fn line(
        &mut self,
        pragma: &mut Pragma,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
        buffer: &mut Vec<Output>,
    ) -> Result<(), Error> {
        self.skip_whitespace(stream, Some(buffer));
        if self.directive(pragma, stream, buffer)? {
            return Ok(());
        }
        if self.ifstates.reading() {
            self.walk(None, None, pragma, stream, buffer)?;
        } else {
            self.skip_to_after_newline(stream, None);
        }
        pragma.clear_line();
        Ok(())
    }

    fn walk(
        &mut self,
        callsite: Option<&Position>,
        in_macro: Option<&str>,
        pragma: &mut Pragma,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
        buffer: &mut Vec<Output>,
    ) -> Result<(), Error> {
        let mut in_quotes = false;
        let mut quote = None;
        let mut just_whitespace = true;
        while let Some(token) = stream.peek() {
            match (token.symbol(), in_quotes) {
                (Symbol::Word(w), false) => {
                    if w == "__EXEC" {
                        return Err(ExecNotSupported::code((**token).clone()));
                    }
                    if Defines::is_unsupported_builtin(w) {
                        return Err(BuiltInNotSupported::code((**token).clone()));
                    }
                    just_whitespace = false;
                    if Some(w.as_str()) != in_macro && self.defines.contains_key(w) {
                        let token = token.clone();
                        self.define_use(
                            callsite.unwrap_or_else(|| token.position()),
                            pragma,
                            stream,
                            buffer,
                        )?;
                    } else {
                        self.output(stream.next().expect("peeked above"), buffer);
                    }
                }
                (Symbol::Directive, false) => {
                    if just_whitespace {
                        if let Some(command) = stream.peek_forward(1) {
                            if [
                                "if", "else", "endif", "ifdef", "ifndef", "define", "undef",
                                "include", "pragma",
                            ]
                            .contains(&command.to_string().as_str())
                            {
                                let _ = stream.peek_backward(1);
                                self.directive(pragma, stream, buffer)?;
                                just_whitespace = true;
                                continue;
                            }
                            let _ = stream.peek_backward(1);
                        }
                    }
                    let token = stream.next().expect("peeked above");
                    if in_macro.is_some()
                    && stream.peek().map_or(false, |t| t.symbol().is_word() && self.defines.contains_key(&t.symbol().to_string()))
                        // check if the # token is from another file, or defined before the callsite, ie not in the root arguments
                        && (token.position().path() != callsite.expect(
                            "callsite should exist if in_macro is some"
                        ).path()
                            || token.position().start().0 < callsite.expect(
                            "callsite should exist if in_macro is some"
                            ).start().0)
                    {
                        self.output(
                            Arc::new(Token::new(Symbol::DoubleQuote, token.position().clone())),
                            buffer,
                        );
                        quote = Some(token.position().clone());
                        continue;
                    }
                    self.output(token, buffer);
                }
                (Symbol::Newline, false) => {
                    just_whitespace = true;
                    self.output(stream.next().expect("peeked above"), buffer);
                    if in_macro.is_none() {
                        return Ok(());
                    }
                }
                (Symbol::DoubleQuote, _) => {
                    just_whitespace = false;
                    in_quotes = !in_quotes;
                    self.output(stream.next().expect("peeked above"), buffer);
                }
                (Symbol::Eoi, _) => {
                    return Ok(());
                }
                (Symbol::Whitespace(_), _) => {
                    self.output(stream.next().expect("peeked above"), buffer);
                }
                (_, _) => {
                    just_whitespace = false;
                    self.output(stream.next().expect("peeked above"), buffer);
                }
            }
            if let Some(quote) = quote {
                self.output(Arc::new(Token::new(Symbol::DoubleQuote, quote)), buffer);
            }
            quote = None;
        }
        Ok(())
    }

    /// Returns the current word, consuming it from the stream
    ///
    /// # Errors
    /// - [`UnexpectedEOF`]: If the stream is at the end of the file
    /// - [`ExpectedIdent`]: If the stream is not at a word
    fn current_word(
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
    ) -> Result<Arc<Token>, Error> {
        if let Some(token) = stream.peek() {
            if token.symbol().is_word() {
                return Ok(stream.next().expect("just checked"));
            }
            if token.symbol().is_eoi() {
                return Err(UnexpectedEOF::code(token.as_ref().clone()));
            }
        }
        Err(ExpectedIdent::code(
            stream.next().expect("just checked").as_ref().clone(),
        ))
    }

    /// Skips whitespace, returning the next word and consuming it from the stream
    ///
    /// # Errors
    /// - [`UnexpectedEOF`]: If the stream is at the end of the file
    /// - [`ExpectedIdent`]: If the stream is not at a word
    fn next_word(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
        buffer: Option<&mut Vec<Output>>,
    ) -> Result<Arc<Token>, Error> {
        self.skip_whitespace(stream, buffer);
        Self::current_word(stream)
    }

    // I might want this later, so for now I am leaving it here
    #[allow(dead_code)]
    /// Skips whitespace, returning the next value and consuming it from the stream
    ///
    /// # Errors
    /// - [`UnexpectedEOF`]: If the stream is at the end of the file
    fn next_value(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
        buffer: Option<&mut Vec<Output>>,
    ) -> Result<Arc<Token>, Error> {
        self.skip_whitespace(stream, buffer);
        if let Some(token) = stream.peek() {
            if token.symbol().is_eoi() {
                return Err(UnexpectedEOF::code(token.as_ref().clone()));
            }
        }
        Ok(stream.next().expect("just checked"))
    }

    fn output(&mut self, token: Arc<Token>, buffer: &mut Vec<Output>) {
        if self.ifstates.reading() && !token.symbol().is_comment() {
            if token.symbol().is_newline()
                && buffer
                    .last()
                    .map_or(false, |t| t.last_symbol().map_or(false, Symbol::is_escape))
            {
                buffer.pop();
                return;
            }
            self.token_count += 1;
            buffer.push(Output::Direct(token));
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub mod tests {
    use std::sync::Arc;

    use hemtt_workspace::reporting::Token;
    use peekmore::{PeekMore, PeekMoreIterator};

    pub fn setup(content: &str) -> PeekMoreIterator<impl Iterator<Item = Arc<Token>>> {
        let workspace = hemtt_workspace::Workspace::builder()
            .memory()
            .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
            .unwrap();
        let test = workspace.join("test.hpp").unwrap();
        test.create_file()
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();
        crate::parse::parse(&test).unwrap().into_iter().peekmore()
    }
}
