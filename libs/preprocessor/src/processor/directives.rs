use hemtt_common::position::Position;
use peekmore::{PeekMore, PeekMoreIterator};
use tracing::{error, trace};

use crate::{
    defines::Defines,
    definition::{Definition, FunctionDefinition},
    ifstate::IfState,
    output::Output,
    symbol::Symbol,
    token::Token,
    Error,
};

use super::Processor;

impl Processor {
    pub(crate) fn directive(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        buffer: &mut Vec<Output>,
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
        buffer: &mut Vec<Output>,
    ) -> Result<(), Error> {
        let command = stream.next().expect("was peeked in directive()");
        let command_word = command.symbol().to_string();
        match (command_word.as_str(), self.ifstates.reading()) {
            ("include", true) => {
                self.directive_include(stream, buffer)?;
                Ok(())
            }
            ("define", true) => {
                self.directive_define(stream)?;
                Ok(())
            }
            ("undef", true) => {
                self.directive_undef(stream)?;
                Ok(())
            }
            ("if", true) => {
                self.directive_if(stream)?;
                Ok(())
            }
            ("ifdef", true) => self.directive_ifdef(true, stream),
            ("ifndef", true) => self.directive_ifdef(false, stream),
            ("if" | "ifdef" | "ifndef", false) => {
                self.ifstates.push(IfState::PassingChild);
                self.skip_to_after_newline(stream, None);
                Ok(())
            }
            ("else", _) => {
                self.ifstates.flip();
                self.expect_nothing_to_newline(stream)?;
                Ok(())
            }
            ("endif", _) => {
                self.ifstates.pop();
                self.expect_nothing_to_newline(stream)?;
                Ok(())
            }
            (_, true) => {
                error!("unknown directive: {}", command_word);
                self.skip_to_after_newline(stream, None);
                Ok(())
            }
            (_, false) => {
                self.skip_to_after_newline(stream, None);
                Ok(())
            }
        }
    }

    pub(crate) fn directive_include(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        buffer: &mut Vec<Output>,
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
        self.defines.insert(&ident_string, (ident, definition));
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

    pub(crate) fn directive_if(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    ) -> Result<(), Error> {
        fn value(defines: &mut Defines, token: Token) -> Result<(Vec<Token>, bool), Error> {
            if let Some((_, definition)) = defines.get(&token, token.source()) {
                if let Definition::Value(tokens) = definition {
                    return Ok((tokens, true));
                } else {
                    panic!("can we do some error handling please");
                }
            }
            Ok((vec![token], false))
        }
        self.skip_whitespace(stream, None);
        let Some(left) = stream.next() else {
            panic!("can we do some error handling please");
        };
        let (left, left_defined) = value(&mut self.defines, left)?;
        self.skip_whitespace(stream, None);
        let mut operator = Vec::with_capacity(2);
        let (right, right_defined) = if stream.peek().map(Token::symbol) == Some(&Symbol::Newline) {
            if !left_defined {
                panic!("can we do some error handling please")
            }
            let equals = Token::new(Symbol::Equals, Position::builtin());
            operator = vec![equals.clone(), equals];
            (
                vec![Token::new(Symbol::Digit(1), Position::builtin())],
                false,
            )
        } else {
            loop {
                let Some(token) = stream.peek() else {
                    // return Err(Error::Code(Box::new(UnexpectedEOF {
                    //     token: Box::new(from),
                    // })));
                    panic!("can we do some error handling please");
                };
                if matches!(token.symbol(), Symbol::Whitespace(_)) {
                    stream.next();
                    break;
                }
                operator.push(token.clone());
                stream.next();
            }
            let Some(right) = stream.next() else {
                // return Err(Error::Code(Box::new(UnexpectedEOF {
                //     token: Box::new(from),
                // })));
                panic!("can we do some error handling please");
            };
            value(&mut self.defines, right)?
        };
        let operator = operator
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<String>();
        let left_string = left
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<String>();
        let right_string = right
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<String>();
        let read = match operator.as_str() {
            "==" => left_string == right_string,
            "!=" => left_string != right_string,
            ">" | ">=" | "<" | "<=" => {
                let left_f64 = left_string.parse::<f64>().map_err(|_| {
                    // Error::Code(Box::new(IfIncompatibleType {
                    //     left: (left.clone(), left_defined),
                    //     operator: operators.clone(),
                    //     right: (right.clone(), right_defined),
                    //     trace: context.trace(),
                    // }))
                    panic!("can we do some error handling please");
                });
                let right_f64 = right_string.parse::<f64>().map_err(|_| {
                    // Error::Code(Box::new(IfIncompatibleType {
                    //     left: (left, left_defined),
                    //     operator: operators,
                    //     right: (right, right_defined),
                    //     trace: context.trace(),
                    // }))
                    panic!("can we do some error handling please");
                });
                match operator.as_str() {
                    ">" => left_f64 > right_f64,
                    ">=" => left_f64 >= right_f64,
                    "<" => left_f64 < right_f64,
                    "<=" => left_f64 <= right_f64,
                    _ => unreachable!(),
                }
            }
            _ => {
                // return Err(Error::Code(Box::new(IfInvalidOperator {
                //     tokens: operators,
                //     trace: context.trace(),
                // })))
                panic!("can we do some error handling please");
            }
        };
        self.ifstates.push(if read {
            IfState::ReadingIf
        } else {
            IfState::PassingIf
        });
        self.expect_nothing_to_newline(stream)?;
        Ok(())
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
        assert_eq!(
            processor.defines.get_test("FLAG").unwrap().1,
            Definition::Unit
        );
    }

    #[test]
    fn test_directive_define_value() {
        let mut stream = tests::setup("#define FLAG 1");
        let mut processor = Processor::default();
        processor.directive(&mut stream, &mut Vec::new()).unwrap();
        assert_eq!(processor.defines.global().len(), 1);
        assert_eq!(
            processor
                .defines
                .get_test("FLAG")
                .unwrap()
                .1
                .as_value()
                .unwrap()[0]
                .symbol(),
            &Symbol::Digit(1)
        );
    }
}
