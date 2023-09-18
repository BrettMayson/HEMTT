use hemtt_common::reporting::{Output, Symbol, Token};
use peekmore::{PeekMore, PeekMoreIterator};

use crate::{
    codes::{
        pe12_include_not_found::IncludeNotFound, pe13_include_not_encased::IncludeNotEncased,
        pe14_include_unexpected_suffix::IncludeUnexpectedSuffix,
        pe15_if_invalid_operator::IfInvalidOperator,
        pe16_if_incompatible_types::IfIncompatibleType, pe2_unexpected_eof::UnexpectedEOF,
        pe3_expected_ident::ExpectedIdent, pe4_unknown_directive::UnknownDirective,
        pe6_change_builtin::ChangeBuiltin, pe7_if_unit_or_function::IfUnitOrFunction,
        pe8_if_undefined::IfUndefined, pw1_redefine::RedefineMacro,
    },
    defines::Defines,
    definition::{Definition, FunctionDefinition},
    ifstate::IfState,
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
                Self::expect_nothing_to_newline(stream)?;
                Ok(())
            }
            ("endif", _) => {
                self.ifstates.pop();
                Self::expect_nothing_to_newline(stream)?;
                Ok(())
            }
            (_, true) => Err(Error::Code(Box::new(UnknownDirective {
                token: Box::new(command),
            }))),
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
            return Err(Error::Code(Box::new(IncludeNotEncased {
                encased_in: if open.symbol().is_word() {
                    None
                } else {
                    Some(open.clone())
                },
                token: Box::new(open),
            })));
        }
        let close = open
            .symbol()
            .matching_enclosure()
            .expect("is_include_enclosure should always have a matching_enclosure");
        let mut path = Vec::new();
        for token in stream.by_ref() {
            let symbol = token.symbol();
            if symbol == &close {
                break;
            }
            if symbol.is_newline() {
                return Err(Error::Code(Box::new(IncludeNotEncased {
                    token: Box::new(token),
                    encased_in: Some(open),
                })));
            }
            if symbol.is_eoi() {
                return Err(Error::Code(Box::new(UnexpectedEOF {
                    token: Box::new(token),
                })));
            }
            path.push(token);
        }

        if let Err(Error::Code(code)) = Self::expect_nothing_to_newline(stream) {
            if let Some(token) = code.token() {
                return Err(Error::Code(Box::new(IncludeUnexpectedSuffix {
                    token: Box::new(token.clone()),
                })));
            }
            return Err(Error::Code(code));
        }

        let current = self
            .files
            .last()
            .expect("root file should always be present");
        let Ok(Some(path)) =
            current.locate(&path.iter().map(ToString::to_string).collect::<String>())
        else {
            return Err(Error::Code(Box::new(IncludeNotFound { token: path })));
        };
        let tokens = crate::parse::parse(&path)?;
        let mut stream = tokens.into_iter().peekmore();
        self.file(&mut stream, buffer)
    }

    pub(crate) fn directive_define(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    ) -> Result<(), Error> {
        let ident = self.next_word(stream, None)?;
        if !ident.symbol().is_word() {
            return Err(Error::Code(Box::new(ExpectedIdent {
                token: Box::new(ident),
            })));
        }
        let ident_string = ident.symbol().to_string();
        let Some(next) = stream.peek() else {
            return Err(Error::Code(Box::new(UnexpectedEOF {
                token: Box::new(Token::new(Symbol::Eoi, ident.position().clone())),
            })));
        };
        if Defines::is_builtin(&ident_string) {
            return Err(Error::Code(Box::new(ChangeBuiltin {
                token: Box::new(ident),
            })));
        }
        if let Some((original, _)) = self.defines.remove(&ident_string) {
            self.warnings.push(Box::new(RedefineMacro {
                token: Box::new(ident.clone()),
                original: Box::new(original),
            }));
        }
        let definition = match next.symbol() {
            Symbol::LeftParenthesis => Definition::Function(FunctionDefinition::new(
                Self::define_read_args(stream)?,
                self.define_read_body(stream),
            )),
            Symbol::Newline | Symbol::Eoi => Definition::Unit,
            _ => Definition::Value(self.define_read_body(stream)),
        };
        self.usage.insert(ident.position().clone(), Vec::new());
        self.defines.insert(&ident_string, (ident, definition));
        Ok(())
    }

    pub(crate) fn directive_undef(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    ) -> Result<(), Error> {
        let ident = self.next_word(stream, None)?;
        if !ident.symbol().is_word() {
            return Err(Error::Code(Box::new(ExpectedIdent {
                token: Box::new(ident),
            })));
        }
        let ident_string = ident.symbol().to_string();
        self.defines.remove(&ident_string);
        Self::expect_nothing_to_newline(stream)
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn directive_if(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    ) -> Result<(), Error> {
        fn value(defines: &mut Defines, token: Token) -> Result<(Vec<Token>, bool), Error> {
            if let Some((_, definition)) = defines.get_with_gen(&token, Some(token.position())) {
                if let Definition::Value(tokens) = definition {
                    return Ok((tokens, true));
                }
                return Err(Error::Code(Box::new(IfUnitOrFunction {
                    token: Box::new(token),
                    defines: defines.clone(),
                })));
            }
            Ok((vec![token], false))
        }
        let left = self.next_value(stream, None)?;
        let (left, left_defined) = value(&mut self.defines, left)?;
        self.skip_whitespace(stream, None);
        let mut operators = Vec::with_capacity(2);
        let (right, right_defined) = if stream.peek().map(Token::symbol) == Some(&Symbol::Newline) {
            let pos = stream.peek().unwrap().position().clone();
            if !left_defined {
                return Err(Error::Code(Box::new(IfUndefined {
                    token: Box::new(left[0].clone()),
                    defines: self.defines.clone(),
                })));
            }
            let equals = Token::new(Symbol::Equals, pos.clone());
            operators = vec![equals.clone(), equals];
            (vec![Token::new(Symbol::Digit(1), pos)], false)
        } else {
            loop {
                let Some(token) = stream.peek() else {
                    return Err(Error::Code(Box::new(UnexpectedEOF {
                        token: Box::new(
                            left.last()
                                .expect("left should exists at this point")
                                .clone(),
                        ),
                    })));
                };
                if matches!(token.symbol(), Symbol::Whitespace(_)) {
                    stream.next();
                    break;
                }
                operators.push(token.clone());
                stream.next();
            }
            let Some(right) = stream.next() else {
                return Err(Error::Code(Box::new(UnexpectedEOF {
                    token: Box::new(
                        left.last()
                            .expect("left should exists at this point")
                            .clone(),
                    ),
                })));
            };
            value(&mut self.defines, right)?
        };
        let operator = operators
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
                    Error::Code(Box::new(IfIncompatibleType {
                        left: (left.clone(), left_defined),
                        operator: operators.clone(),
                        right: (right.clone(), right_defined),
                    }))
                })?;
                let right_f64 = right_string.parse::<f64>().map_err(|_| {
                    Error::Code(Box::new(IfIncompatibleType {
                        left: (left, left_defined),
                        operator: operators,
                        right: (right, right_defined),
                    }))
                })?;
                match operator.as_str() {
                    ">" => left_f64 > right_f64,
                    ">=" => left_f64 >= right_f64,
                    "<" => left_f64 < right_f64,
                    "<=" => left_f64 <= right_f64,
                    _ => unreachable!(),
                }
            }
            _ => {
                return Err(Error::Code(Box::new(IfInvalidOperator {
                    tokens: operators,
                })))
            }
        };
        self.ifstates.push(if read {
            IfState::ReadingIf
        } else {
            IfState::PassingIf
        });
        Self::expect_nothing_to_newline(stream)?;
        Ok(())
    }

    pub(crate) fn directive_ifdef(
        &mut self,
        outcome: bool,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
    ) -> Result<(), Error> {
        let ident = self.next_word(stream, None)?;
        if !ident.symbol().is_word() {
            return Err(Error::Code(Box::new(ExpectedIdent {
                token: Box::new(ident),
            })));
        }
        let ident_string = ident.symbol().to_string();
        self.ifstates
            .push_if(self.defines.contains_key(&ident_string) == outcome);
        Self::expect_nothing_to_newline(stream)
    }
}

#[cfg(test)]
mod tests {
    use hemtt_common::reporting::Symbol;

    use crate::{
        definition::Definition,
        processor::{tests, Processor},
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
