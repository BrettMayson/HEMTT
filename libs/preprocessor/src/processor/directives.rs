use peekmore::{PeekMore, PeekMoreIterator};
use tracing::{error, trace};

use crate::{
    definition::{Definition, FunctionDefinition},
    ifstate::IfState,
    symbol::Symbol,
    token::Token,
    Error,
};

use super::Processor;

impl Processor {
    pub(crate) fn directive(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        buffer: &mut Vec<Token>,
    ) -> Result<bool, Error> {
        if let Some(token) = stream.peek() {
            if token.symbol().is_directive() {
                stream.next();
                if let Some(command) = stream.peek() {
                    if command.symbol().is_word() {
                        self.directive_command(stream, buffer)?;
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    pub(crate) fn directive_command(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        buffer: &mut Vec<Token>,
    ) -> Result<(), Error> {
        let command = stream.next().expect("was peeked in directive()");
        let command_word = command.symbol().to_string();
        match (command_word.as_str(), self.ifstates.reading()) {
            ("include", true) => {
                self.directive_include(stream, buffer)?;
                Ok(())
            }
            ("define", true) => {
                error!("define a macro");
                self.directive_define(stream)?;
                Ok(())
            }
            ("undef", true) => {
                error!("undefine a macro");
                self.directive_undef(stream)?;
                Ok(())
            }
            ("if", true) => {
                error!("if");
                Ok(())
            }
            ("ifdef", true) => {
                trace!("ifdef");
                self.directive_ifdef(true, stream)
            }
            ("ifndef", true) => {
                trace!("ifndef");
                self.directive_ifdef(false, stream)
            }
            ("if" | "ifdef" | "ifndef", false) => {
                trace!("skip if*");
                self.ifstates.push(IfState::PassingChild);
                self.skip_to_after_newline(stream, None);
                Ok(())
            }
            ("else", _) => {
                trace!("else");
                self.ifstates.flip();
                self.expect_nothing_to_newline(stream)?;
                Ok(())
            }
            ("endif", _) => {
                trace!("endif");
                self.ifstates.pop();
                self.expect_nothing_to_newline(stream)?;
                Ok(())
            }
            (_, _) => {
                error!("unknown directive: {}", command_word);
                Ok(())
            }
        }
    }

    pub(crate) fn directive_include(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        buffer: &mut Vec<Token>,
    ) -> Result<(), Error> {
        self.skip_whitespace(stream, None);
        let open = stream.next().expect("was peeked in directive()");
        if !open.symbol().is_include_enclosure() {
            panic!("you can't use {:?} for include", open.symbol());
        }
        let close = open
            .symbol()
            .matching_enclosure()
            .expect("is_include_enclosure should always have a matching_enclosure");
        let mut path = String::new();
        for token in stream.by_ref() {
            if token.symbol() == &close {
                break;
            }
            path.push_str(token.to_string().as_str());
        }

        let current = self
            .files
            .last()
            .expect("root file should always be present");
        let Ok(Some(path)) = current.locate(&path) else {
            panic!("can we do some error handling please");
        };
        let tokens = crate::parse::parse(&path)?;
        let mut stream = tokens.into_iter().peekmore();
        self.file(&mut stream, buffer)
    }

    pub(crate) fn directive_define(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    ) -> Result<(), Error> {
        let Some(ident) = self.next_word(stream, None) else {
            panic!("can we do some error handling please");
        };
        let ident_string = ident.symbol().to_string();
        let Some(next) = stream.peek() else {
            panic!("can we do some error handling please");
        };
        if self.defines.contains_key(&ident_string) {
            error!("redefining macro: {}", ident_string);
        }
        self.defines.remove(&ident_string);
        let definition = match next.symbol() {
            Symbol::LeftParenthesis => Definition::Function(FunctionDefinition::new(
                self.define_read_args(stream)?,
                self.define_read_body(stream)?,
            )),
            Symbol::Newline | Symbol::Eoi => Definition::Unit,
            _ => Definition::Value(self.define_read_body(stream)?),
        };
        self.defines.insert(ident_string, (ident, definition));
        Ok(())
    }

    pub(crate) fn directive_undef(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    ) -> Result<(), Error> {
        let Some(ident) = self.next_word(stream, None) else {
            panic!("can we do some error handling please");
        };
        let ident_string = ident.symbol().to_string();
        self.defines.remove(&ident_string);
        self.expect_nothing_to_newline(stream)
    }

    pub(crate) fn directive_ifdef(
        &mut self,
        outcome: bool,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    ) -> Result<(), Error> {
        let Some(ident) = self.next_word(stream, None) else {
            panic!("can we do some error handling please");
        };
        let ident_string = ident.symbol().to_string();
        self.ifstates
            .push_if(self.defines.contains_key(&ident_string) == outcome);
        self.expect_nothing_to_newline(stream)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        definition::Definition,
        processor::{tests, Processor},
        symbol::Symbol,
    };

    #[test]
    fn test_directive_define_unit() {
        let mut stream = tests::setup("#define FLAG");
        let mut processor = Processor::default();
        processor.directive(&mut stream, &mut Vec::new()).unwrap();
        assert_eq!(processor.defines.global().len(), 1);
        assert_eq!(processor.defines.get("FLAG").unwrap().1, Definition::Unit);
    }

    #[test]
    fn test_directive_define_value() {
        let mut stream = tests::setup("#define FLAG 1");
        let mut processor = Processor::default();
        processor.directive(&mut stream, &mut Vec::new()).unwrap();
        assert_eq!(processor.defines.global().len(), 1);
        assert_eq!(
            processor.defines.get("FLAG").unwrap().1.as_value().unwrap()[0].symbol(),
            &Symbol::Digit(1)
        );
    }
}
