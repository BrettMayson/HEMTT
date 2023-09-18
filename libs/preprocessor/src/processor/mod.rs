use std::collections::HashMap;

use hemtt_common::position::Position;
use hemtt_common::reporting::{Code, Output, Processed, Symbol, Token};
use hemtt_common::workspace::WorkspacePath;
use peekmore::{PeekMore, PeekMoreIterator};

use crate::codes::pe2_unexpected_eof::UnexpectedEOF;
use crate::codes::pe3_expected_ident::ExpectedIdent;
use crate::defines::Defines;
use crate::ifstate::IfStates;
use crate::Error;

mod defines;
mod directives;
mod whitespace;

#[derive(Default)]
/// Arma 3 Preprocessor
pub struct Processor {
    ifstates: IfStates,
    defines: Defines,

    files: Vec<WorkspacePath>,

    pub(crate) token_count: usize,

    /// Map of token usage to definition
    /// (token, definition)
    pub(crate) declarations: HashMap<Position, Position>,

    /// Map of token definition to usage
    /// (definition, usages)
    pub(crate) usage: HashMap<Position, Vec<Position>>,

    /// Warnings
    pub(crate) warnings: Vec<Box<dyn Code>>,
}

impl Processor {
    /// Preprocess a file
    ///
    /// # Errors
    /// See [`Error`]
    pub fn run(path: &WorkspacePath) -> Result<Processed, Error> {
        let mut processor = Self::default();

        processor.files.push(path.clone());

        let tokens = crate::parse::parse(path)?;
        let mut buffer = Vec::with_capacity(tokens.len());
        let mut stream = tokens.into_iter().peekmore();

        processor.file(&mut stream, &mut buffer)?;
        Processed::new(
            buffer,
            processor.usage,
            processor.declarations,
            processor.warnings,
        )
        .map_err(Into::into)
    }

    fn file(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        buffer: &mut Vec<Output>,
    ) -> Result<(), Error> {
        loop {
            let first = stream.peek();
            if first.is_none() || first.expect("just checked").symbol().is_eoi() {
                return Ok(());
            }
            self.line(stream, buffer)?;
        }
    }

    fn line(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        buffer: &mut Vec<Output>,
    ) -> Result<(), Error> {
        self.skip_whitespace(stream, Some(buffer));
        if self.directive(stream, buffer)? {
            return Ok(());
        }
        if self.ifstates.reading() {
            self.walk(None, None, stream, buffer)?;
        } else {
            self.skip_to_after_newline(stream, None);
        }
        Ok(())
    }

    fn walk(
        &mut self,
        callsite: Option<&Position>,
        in_macro: Option<&str>,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        buffer: &mut Vec<Output>,
    ) -> Result<(), Error> {
        let mut in_quotes = false;
        let mut quote = None;
        while let Some(token) = stream.peek() {
            match (token.symbol(), in_quotes) {
                (Symbol::Word(w), false) => {
                    if Some(w.as_str()) != in_macro && self.defines.contains_key(w) {
                        let token = token.clone();
                        self.define_use(
                            callsite.unwrap_or_else(|| token.position()),
                            stream,
                            buffer,
                        )?;
                    } else {
                        self.output(stream.next().expect("peeked above"), buffer);
                    }
                }
                (Symbol::Directive, false) => {
                    let token = stream.next().expect("peeked above");
                    if in_macro.is_some() && stream.peek().map_or(false, |t| t.symbol().is_word()) {
                        self.output(
                            Token::new(Symbol::DoubleQuote, token.position().clone()),
                            buffer,
                        );
                        quote = Some(token.position().clone());
                        continue;
                    }
                    self.output(token, buffer);
                }
                (Symbol::Newline, false) => {
                    self.output(stream.next().expect("peeked above"), buffer);
                    if in_macro.is_none() {
                        return Ok(());
                    }
                }
                (Symbol::DoubleQuote, _) => {
                    in_quotes = !in_quotes;
                    self.output(stream.next().expect("peeked above"), buffer);
                }
                (Symbol::Eoi, _) => {
                    return Ok(());
                }
                (_, _) => {
                    self.output(stream.next().expect("peeked above"), buffer);
                }
            }
            if let Some(quote) = quote {
                self.output(Token::new(Symbol::DoubleQuote, quote), buffer);
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
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    ) -> Result<Token, Error> {
        if let Some(token) = stream.peek() {
            if token.symbol().is_word() {
                return Ok(stream.next().expect("just checked"));
            }
            if token.symbol().is_eoi() {
                return Err(Error::Code(Box::new(UnexpectedEOF {
                    token: Box::new(token.clone()),
                })));
            }
        }
        Err(Error::Code(Box::new(ExpectedIdent {
            token: Box::new(stream.next().expect("just checked")),
        })))
    }

    /// Skips whitespace, returning the next word and consuming it from the stream
    ///
    /// # Errors
    /// - [`UnexpectedEOF`]: If the stream is at the end of the file
    /// - [`ExpectedIdent`]: If the stream is not at a word
    fn next_word(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        buffer: Option<&mut Vec<Output>>,
    ) -> Result<Token, Error> {
        self.skip_whitespace(stream, buffer);
        Self::current_word(stream)
    }

    /// Skips whitespace, returning the next value and consuming it from the stream
    ///
    /// # Errors
    /// - [`UnexpectedEOF`]: If the stream is at the end of the file
    fn next_value(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        buffer: Option<&mut Vec<Output>>,
    ) -> Result<Token, Error> {
        self.skip_whitespace(stream, buffer);
        if let Some(token) = stream.peek() {
            if token.symbol().is_eoi() {
                return Err(Error::Code(Box::new(UnexpectedEOF {
                    token: Box::new(token.clone()),
                })));
            }
        }
        Ok(stream.next().expect("just checked"))
    }

    fn output(&mut self, token: Token, buffer: &mut Vec<Output>) {
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
pub mod tests {
    use hemtt_common::reporting::Token;
    use peekmore::{PeekMore, PeekMoreIterator};

    pub fn setup(content: &str) -> PeekMoreIterator<impl Iterator<Item = Token>> {
        let workspace = hemtt_common::workspace::Workspace::builder()
            .memory()
            .finish()
            .unwrap();
        let test = workspace.join("test.hpp").unwrap();
        test.create_file()
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();
        crate::parse::parse(&test).unwrap().into_iter().peekmore()
    }

    // pub fn setup(content: &str) -> Processed {
    //     let workspace = hemtt_common::workspace::Workspace::builder()
    //         .memory()
    //         .finish()
    //         .unwrap();
    //     let test = workspace.join("test.hpp").unwrap();
    //     test.create_file()
    //         .unwrap()
    //         .write_all(content.as_bytes())
    //         .unwrap();
    //     Processed::new(&test).unwrap()
    // }

    // #[test]
    // fn simple_define() {
    //     let processed = setup("#define number 1\nvalue = number;");
    //     assert_eq!(processed.as_string(), "value = 1;");
    //     let mapping = processed.mapping(9);
    //     println!("{:?}", mapping);
    //     println!("{:?}", processed.usage);
    // }
}
