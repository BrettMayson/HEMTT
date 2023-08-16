use hemtt_common::workspace::WorkspacePath;
use peekmore::{PeekMore, PeekMoreIterator};

use crate::defines::Defines;
use crate::ifstate::IfStates;
use crate::processed::Processed;
use crate::symbol::Symbol;
use crate::token::Token;
use crate::Error;

mod defines;
mod directives;
mod whitespace;

#[derive(Default)]
pub struct Processor {
    ifstates: IfStates,
    defines: Defines,

    files: Vec<WorkspacePath>,
    processed: Processed,
}

impl Processor {
    pub fn run(path: &WorkspacePath) -> Result<Processed, Error> {
        let mut processor = Self::default();

        processor.files.push(path.clone());

        let tokens = crate::parse::parse(path)?;
        let mut buffer = Vec::with_capacity(tokens.len());
        let mut stream = tokens.into_iter().peekmore();

        processor.file(&mut stream, &mut buffer)?;
        processor.processed.tokens = buffer;
        Ok(processor.processed)
    }

    fn file(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        buffer: &mut Vec<Token>,
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
        buffer: &mut Vec<Token>,
    ) -> Result<(), Error> {
        self.skip_whitespace(stream, Some(buffer));
        if self.directive(stream, buffer)? {
            return Ok(());
        }
        self.walk(None, stream, buffer)?;
        Ok(())
    }

    fn walk(
        &mut self,
        in_macro: Option<&str>,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        buffer: &mut Vec<Token>,
    ) -> Result<(), Error> {
        let mut in_quotes = false;
        let mut quote = None;
        while let Some(token) = stream.peek() {
            println!("walking: {:?}", token.symbol());
            match (token.symbol(), in_quotes) {
                (Symbol::Word(w), false) => {
                    if Some(w.as_str()) != in_macro && self.defines.contains_key(w) {
                        self.define_use(stream, buffer)?;
                    } else {
                        self.output(stream.next().expect("peeked above"), buffer);
                    }
                }
                (Symbol::Directive, false) => {
                    let token = stream.next().expect("peeked above");
                    if in_macro.is_some() && stream.peek().map_or(false, |t| t.symbol().is_word()) {
                        self.output(
                            Token::new(Symbol::DoubleQuote, token.source().to_owned()),
                            buffer,
                        );
                        quote = Some(token.source().clone());
                        continue;
                    }
                    self.output(stream.next().expect("peeked above"), buffer);
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

    // Returns the current word, consuming it from the stream
    // If the stream is not at a word, returns None
    fn current_word(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    ) -> Option<Token> {
        if let Some(token) = stream.peek() {
            if token.symbol().is_word() {
                return Some(stream.next().expect("just checked"));
            }
        }
        None
    }

    /// Skips whitespace, returning the next word and consuming it from the stream
    /// If there is no word, returns None
    fn next_word(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        buffer: Option<&mut Vec<Token>>,
    ) -> Option<Token> {
        self.skip_whitespace(stream, buffer);
        self.current_word(stream)
    }

    fn output(&mut self, token: Token, buffer: &mut Vec<Token>) {
        if self.ifstates.reading() && !token.symbol().is_comment() {
            if buffer
                .last()
                .map(|t| t.symbol().is_escape())
                .unwrap_or(false)
            {
                buffer.pop();
                return;
            }
            println!("output: {:?}", token.symbol());
            buffer.push(token);
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use peekmore::{PeekMore, PeekMoreIterator};

    use crate::token::Token;

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
}
