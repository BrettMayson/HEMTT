use std::rc::Rc;

use hemtt_common::{
    position::Position,
    reporting::{Output, Symbol, Token},
};
use peekmore::{PeekMore, PeekMoreIterator};
use tracing::debug;

use crate::{
    codes::{
        pe12_include_not_found::IncludeNotFound, pe13_include_not_encased::IncludeNotEncased,
        pe14_include_unexpected_suffix::IncludeUnexpectedSuffix,
        pe15_if_invalid_operator::IfInvalidOperator,
        pe16_if_incompatible_types::IfIncompatibleType, pe19_pragma_unknown::PragmaUnknown,
        pe20_pragma_invalid_scope::PragmaInvalidScope, pe23_if_has_include::IfHasInclude,
        pe2_unexpected_eof::UnexpectedEOF, pe3_expected_ident::ExpectedIdent,
        pe4_unknown_directive::UnknownDirective, pe6_change_builtin::ChangeBuiltin,
        pe7_if_unit_or_function::IfUnitOrFunction, pe8_if_undefined::IfUndefined,
        pw1_redefine::RedefineMacro,
    },
    defines::Defines,
    definition::{Definition, FunctionDefinition},
    ifstate::IfState,
    processor::pragma::Flag,
    Error,
};

use super::{
    pragma::{Pragma, Scope},
    Processor,
};

impl Processor {
    pub(crate) fn directive(
        &mut self,
        pragma: &mut Pragma,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Rc<Token>>>,
        buffer: &mut Vec<Output>,
    ) -> Result<bool, Error> {
        if let Some(token) = stream.peek() {
            if token.symbol().is_directive() {
                stream.next();
                if let Some(command) = stream.peek() {
                    if command.symbol().is_word() {
                        self.directive_command(pragma, stream, buffer)?;
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    pub(crate) fn directive_command(
        &mut self,
        pragma: &mut Pragma,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Rc<Token>>>,
        buffer: &mut Vec<Output>,
    ) -> Result<(), Error> {
        let command = stream.next().expect("was peeked in directive()");
        let command_word = command.symbol().to_string();
        match (command_word.as_str(), self.ifstates.reading()) {
            ("include", true) => {
                self.directive_include(pragma, stream, buffer)?;
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
                self.directive_if(pragma, command, stream)?;
                Ok(())
            }
            ("ifdef", true) => self.directive_ifdef(command, true, stream),
            ("ifndef", true) => self.directive_ifdef(command, false, stream),
            ("if" | "ifdef" | "ifndef", false) => {
                self.ifstates.push(IfState::PassingChild(command));
                self.skip_to_after_newline(stream, None);
                Ok(())
            }
            ("else", _) => {
                self.ifstates.flip(command)?;
                Self::expect_nothing_to_newline(stream)?;
                Ok(())
            }
            ("endif", _) => {
                self.ifstates.pop();
                Self::expect_nothing_to_newline(stream)?;
                Ok(())
            }
            ("pragma", true) => {
                if self.next_word(stream, None)?.to_string() != "hemtt" {
                    self.skip_to_after_newline(stream, None);
                    return Ok(());
                }
                let command = self.next_word(stream, None)?;
                match command.to_string().as_str() {
                    "suppress" => {
                        let (code, scope) = self.read_pragma(&command, pragma, stream)?;
                        pragma.suppress(&code, scope)?;
                    }
                    "flag" => {
                        let (code, scope) = self.read_pragma(&command, pragma, stream)?;
                        pragma.flag(&code, scope)?;
                    }
                    _ => {
                        return Err(Error::Code(Box::new(PragmaUnknown {
                            token: Box::new(command.as_ref().clone()),
                        })))
                    }
                }
                Ok(())
            }
            (_, false) => {
                self.skip_to_after_newline(stream, None);
                Ok(())
            }
            (_, true) => Err(Error::Code(Box::new(UnknownDirective {
                token: Box::new(command.as_ref().clone()),
            }))),
        }
    }

    pub(crate) fn read_pragma(
        &mut self,
        command: &Rc<Token>,
        pragma: &Pragma,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Rc<Token>>>,
    ) -> Result<(Rc<Token>, Scope), Error> {
        let code = self.next_word(stream, None)?;
        let mut hit_end = false;
        let scope_token = self.next_word(stream, None).unwrap_or_else(|_| {
            hit_end = true;
            Rc::new(Token::new(
                Symbol::Word("line".to_string()),
                command.position().clone(),
            ))
        });
        let Ok(scope) = Scope::try_from(scope_token.to_string().as_str()) else {
            return Err(Error::Code(Box::new(PragmaInvalidScope {
                token: Box::new(scope_token.as_ref().clone()),
                root: pragma.root,
            })));
        };
        if !hit_end {
            Self::expect_nothing_to_newline(stream)?;
        };
        Ok((code, scope))
    }

    #[allow(clippy::needless_pass_by_ref_mut)]
    pub(crate) fn directive_include(
        &mut self,
        pragma: &mut Pragma,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Rc<Token>>>,
        buffer: &mut Vec<Output>,
    ) -> Result<(), Error> {
        self.skip_whitespace(stream, None);
        let open = stream.next().expect("was peeked in directive()");
        if !open.symbol().is_include_enclosure() {
            return Err(Error::Code(Box::new(IncludeNotEncased {
                encased_in: if open.symbol().is_word() {
                    None
                } else {
                    Some(open.as_ref().clone())
                },
                token: Box::new(open.as_ref().clone()),
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
                    token: Box::new(token.as_ref().clone()),
                    encased_in: Some(open.as_ref().clone()),
                })));
            }
            if symbol.is_eoi() {
                return Err(Error::Code(Box::new(UnexpectedEOF {
                    token: Box::new(token.as_ref().clone()),
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
            current.locate(&path.iter().map(|t| t.to_string()).collect::<String>())
        else {
            return Err(Error::Code(Box::new(IncludeNotFound::new(path))));
        };
        let tokens = crate::parse::parse(&path)?;
        self.files.push(path);
        let mut stream = tokens.into_iter().peekmore();
        let ret = self.file(&mut pragma.child(), &mut stream, buffer);
        self.files.pop();
        ret
    }

    pub(crate) fn directive_define(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Rc<Token>>>,
    ) -> Result<(), Error> {
        let ident = self.next_word(stream, None)?;
        if !ident.symbol().is_word() {
            return Err(Error::Code(Box::new(ExpectedIdent {
                token: Box::new(ident.as_ref().clone()),
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
                token: Box::new(ident.as_ref().clone()),
            })));
        }
        if let Some((original, _)) = self.defines.remove(&ident_string) {
            self.warnings.push(Box::new(RedefineMacro {
                token: Box::new(ident.as_ref().clone()),
                original: Box::new(original.as_ref().clone()),
            }));
        }
        let definition = match next.symbol() {
            Symbol::LeftParenthesis => Definition::Function({
                let args = Self::define_read_args(stream)?;
                let body = self.define_read_body(stream);
                let position = if body.first().is_some() {
                    Position::new(
                        *body.first().unwrap().position().start(),
                        *body.last().unwrap().position().end(),
                        ident.position().path().clone(),
                    )
                } else {
                    ident.position().clone()
                };
                FunctionDefinition::new(position, args, body)
            }),
            Symbol::Newline | Symbol::Eoi => Definition::Unit,
            _ => Definition::Value(self.define_read_body(stream)),
        };
        #[cfg(feature = "lsp")]
        self.usage.insert(ident.position().clone(), Vec::new());
        self.defines.insert(&ident_string, (ident, definition));
        Ok(())
    }

    pub(crate) fn directive_undef(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Rc<Token>>>,
    ) -> Result<(), Error> {
        let ident = self.next_word(stream, None)?;
        if !ident.symbol().is_word() {
            return Err(Error::Code(Box::new(ExpectedIdent {
                token: Box::new(ident.as_ref().clone()),
            })));
        }
        let ident_string = ident.symbol().to_string();
        self.defines.remove(&ident_string);
        Self::expect_nothing_to_newline(stream)
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn directive_if(
        &mut self,
        pragma: &Pragma,
        command: Rc<Token>,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Rc<Token>>>,
    ) -> Result<(), Error> {
        fn value(defines: &mut Defines, token: Rc<Token>) -> Result<(Vec<Rc<Token>>, bool), Error> {
            if let Some((_, definition)) = defines.get_with_gen(&token, Some(token.position())) {
                if let Definition::Value(tokens) = definition {
                    return Ok((tokens, true));
                }
                return Err(Error::Code(Box::new(IfUnitOrFunction::new(
                    Box::new(token.as_ref().clone()),
                    &defines.clone(),
                ))));
            }
            Ok((vec![token], false))
        }
        let left = self.next_value(stream, None)?;
        if &Symbol::Word(String::from("__has_include")) == left.symbol() {
            if pragma.is_flagged(&Flag::Pe23IgnoreIfHasInclude) {
                debug!(
                    "ignoring __has_include due to pragma flag, this config will not be rapified"
                );
                self.no_rapify = true;
                self.ifstates.push_if(command, false);
                self.skip_to_after_newline(stream, None);
                return Ok(());
            }
            return Err(Error::Code(Box::new(IfHasInclude {
                token: Box::new(left.as_ref().clone()),
            })));
        }
        let (left, left_defined) = value(&mut self.defines, left)?;
        self.skip_whitespace(stream, None);
        let mut operators = Vec::with_capacity(2);
        let (right, right_defined) = if stream.peek().map(|t| t.symbol()) == Some(&Symbol::Newline)
        {
            let pos = stream.peek().unwrap().position().clone();
            if !left_defined {
                return Err(Error::Code(Box::new(IfUndefined::new(
                    Box::new(left[0].as_ref().clone()),
                    &self.defines,
                ))));
            }
            let equals = Rc::new(Token::new(Symbol::Equals, pos.clone()));
            operators = vec![equals.clone(), equals];
            (vec![Rc::new(Token::new(Symbol::Digit(1), pos))], false)
        } else {
            loop {
                let Some(token) = stream.peek() else {
                    return Err(Error::Code(Box::new(UnexpectedEOF {
                        token: Box::new(
                            left.last()
                                .expect("left should exists at this point")
                                .as_ref()
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
                            .as_ref()
                            .clone(),
                    ),
                })));
            };
            value(&mut self.defines, right)?
        };
        let operator = operators.iter().map(|t| t.to_string()).collect::<String>();
        let left_string = left.iter().map(|t| t.to_string()).collect::<String>();
        let right_string = right.iter().map(|t| t.to_string()).collect::<String>();
        let read = match operator.as_str() {
            "==" => left_string == right_string,
            "!=" => left_string != right_string,
            ">" | ">=" | "<" | "<=" => {
                let Ok(left_f64) = left_string.parse::<f64>() else {
                    return Err(Error::Code(Box::new(IfIncompatibleType::new(
                        (left, left_defined),
                        operators,
                        (right, right_defined),
                    ))));
                };
                let Ok(right_f64) = right_string.parse::<f64>() else {
                    return Err(Error::Code(Box::new(IfIncompatibleType::new(
                        (left, left_defined),
                        operators,
                        (right, right_defined),
                    ))));
                };
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
                    tokens: operators.iter().map(|t| t.as_ref().clone()).collect(),
                })))
            }
        };
        self.ifstates.push_if(command, read);
        Self::expect_nothing_to_newline(stream)?;
        Ok(())
    }

    pub(crate) fn directive_ifdef(
        &mut self,
        command: Rc<Token>,
        outcome: bool,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Rc<Token>>>,
    ) -> Result<(), Error> {
        let ident = self.next_word(stream, None)?;
        if !ident.symbol().is_word() {
            return Err(Error::Code(Box::new(ExpectedIdent {
                token: Box::new(ident.as_ref().clone()),
            })));
        }
        let ident_string = ident.symbol().to_string();
        self.ifstates
            .push_if(command, self.defines.contains_key(&ident_string) == outcome);
        Self::expect_nothing_to_newline(stream)
    }
}

#[cfg(test)]
mod tests {
    use hemtt_common::reporting::Symbol;

    use crate::{
        definition::Definition,
        processor::{pragma::Pragma, tests, Processor},
    };

    #[test]
    fn test_directive_define_unit() {
        let mut stream = tests::setup("#define FLAG");
        let mut processor = Processor::default();
        processor
            .directive(&mut Pragma::root(), &mut stream, &mut Vec::new())
            .unwrap();
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
        processor
            .directive(&mut Pragma::root(), &mut stream, &mut Vec::new())
            .unwrap();
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
