use std::sync::Arc;

use hemtt_workspace::{
    path::LocateResult,
    position::Position,
    reporting::{Definition, FunctionDefinition, Output, Symbol, Token},
};
use peekmore::{PeekMore, PeekMoreIterator};
use tracing::debug;

use crate::{
    Error,
    codes::{
        pe2_unexpected_eof::UnexpectedEOF, pe3_expected_ident::ExpectedIdent,
        pe4_unknown_directive::UnknownDirective, pe6_change_builtin::ChangeBuiltin,
        pe7_if_unit_or_function::IfUnitOrFunction, pe8_if_undefined::IfUndefined,
        pe12_include_not_found::IncludeNotFound, pe13_include_not_encased::IncludeNotEncased,
        pe14_include_unexpected_suffix::IncludeUnexpectedSuffix,
        pe15_if_invalid_operator::IfInvalidOperator,
        pe16_if_incompatible_types::IfIncompatibleType, pe19_pragma_unknown::PragmaUnknown,
        pe20_pragma_invalid_scope::PragmaInvalidScope, pe23_if_has_include::IfHasInclude,
        pe27_unexpected_endif::UnexpectedEndif, pe28_unexpected_else::UnexpectedElse,
        pw1_redefine::RedefineMacro, pw4_include_case::IncludeCase,
    },
    defines::{DefineSource, Defines},
    ifstate::IfState,
    processor::pragma::Flag,
};

use super::{
    Processor,
    pragma::{Pragma, Scope},
};

impl Processor {
    pub(crate) fn directive(
        &mut self,
        pragma: &mut Pragma,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
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
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
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
                if self.ifstates.is_empty() {
                    return Err(UnexpectedElse::code(command.as_ref().clone()));
                }
                self.ifstates.flip(command)?;
                Self::expect_nothing_to_newline(stream)?;
                Ok(())
            }
            ("endif", _) => {
                if self.ifstates.is_empty() {
                    return Err(UnexpectedEndif::code(command.as_ref().clone()));
                }
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
                    _ => return Err(PragmaUnknown::code(command.as_ref().clone())),
                }
                Ok(())
            }
            (_, false) => {
                self.skip_to_after_newline(stream, None);
                Ok(())
            }
            (_, true) => Err(UnknownDirective::code(command.as_ref().clone())),
        }
    }

    pub(crate) fn read_pragma(
        &mut self,
        command: &Arc<Token>,
        pragma: &Pragma,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
    ) -> Result<(Arc<Token>, Scope), Error> {
        let code = self.next_word(stream, None)?;
        let mut hit_end = false;
        let scope_token = self.next_word(stream, None).unwrap_or_else(|_| {
            hit_end = true;
            Arc::new(Token::new(
                Symbol::Word("line".to_string()),
                command.position().clone(),
            ))
        });
        let Ok(scope) = Scope::try_from(scope_token.to_string().as_str()) else {
            return Err(PragmaInvalidScope::code(
                scope_token.as_ref().clone(),
                pragma.root,
            ));
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
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
        buffer: &mut Vec<Output>,
    ) -> Result<(), Error> {
        self.skip_whitespace(stream, None);
        let open = stream.next().expect("was peeked in directive()");
        if !open.symbol().is_include_enclosure() {
            return Err(IncludeNotEncased::code(
                open.as_ref().clone(),
                Vec::new(),
                if open.symbol().is_word() {
                    None
                } else {
                    Some(open.as_ref().clone())
                },
            ));
        }
        let close = open
            .symbol()
            .matching_enclosure()
            .expect("is_include_enclosure should always have a matching_enclosure");
        let mut path_tokens = Vec::new();
        for token in stream.by_ref() {
            let symbol = token.symbol();
            if symbol == &close {
                break;
            }
            if symbol.is_newline() {
                return Err(IncludeNotEncased::code(
                    token.as_ref().clone(),
                    path_tokens,
                    Some(open.as_ref().clone()),
                ));
            }
            if symbol.is_eoi() {
                return Err(UnexpectedEOF::code(token.as_ref().clone()));
            }
            path_tokens.push(token);
        }

        if let Err(Error::Code(code)) = Self::expect_nothing_to_newline(stream) {
            if let Some(token) = code.token() {
                return Err(IncludeUnexpectedSuffix::code(token.clone()));
            }
            return Err(Error::Code(code));
        }

        let current = self
            .file_stack
            .last()
            .expect("root file should always be present");
        let path = {
            let Ok(Some(LocateResult {
                path: found_path,
                case_mismatch,
            })) = current.locate(
                &path_tokens
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<String>(),
            )
            else {
                return Err(IncludeNotFound::code(path_tokens));
            };
            if let Some(case_mismatch) = case_mismatch {
                self.warnings.push(Arc::new(IncludeCase::new(
                    path_tokens.iter().map(|t| t.as_ref().clone()).collect(),
                    case_mismatch,
                )));
            }
            found_path
        };
        let tokens = crate::parse::file(&path)?;
        self.add_include(path, path_tokens)?;
        let mut stream = tokens.into_iter().peekmore();
        let ret = self.file(&mut pragma.child(), &mut stream, buffer);
        self.file_stack.pop();
        ret
    }

    pub(crate) fn directive_define(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
    ) -> Result<(), Error> {
        let ident = self.next_word(stream, None)?;
        if !ident.symbol().is_word() {
            return Err(ExpectedIdent::code(ident.as_ref().clone()));
        }
        let ident_string = ident.symbol().to_string();
        let Some(next) = stream.peek() else {
            return Err(UnexpectedEOF::code(Token::new(
                Symbol::Eoi,
                ident.position().clone(),
            )));
        };
        if Defines::is_builtin(&ident_string) {
            return Err(ChangeBuiltin::code(ident.as_ref().clone()));
        }
        if let Some((original, _, DefineSource::Source(file_stack))) =
            self.defines.remove(&ident_string)
        {
            self.warnings.push(Arc::new(RedefineMacro::new(
                Box::new(ident.as_ref().clone()),
                self.file_stack.clone(),
                Box::new(original.as_ref().clone()),
                file_stack,
            )));
        }
        let definition = match next.symbol() {
            Symbol::LeftParenthesis => Definition::Function({
                let args = Self::define_read_args(stream)?;
                let body = self.define_read_body(stream);
                let position = if body.is_empty() {
                    ident.position().clone()
                } else {
                    Position::new(
                        *body
                            .first()
                            .expect("must exist because of the if")
                            .position()
                            .start(),
                        *body
                            .last()
                            .expect("must exist because of the if")
                            .position()
                            .end(),
                        ident.position().path().clone(),
                    )
                };
                Arc::new(FunctionDefinition::new(position, args, body))
            }),
            Symbol::Newline | Symbol::Eoi => Definition::Unit,
            _ => Definition::Value(Arc::new(self.define_read_body(stream))),
        };
        #[cfg(feature = "lsp")]
        self.usage.insert(ident.position().clone(), Vec::new());
        self.macros
            .entry(ident_string.clone())
            .or_default()
            .push((ident.position().clone(), definition.clone()));
        self.defines.insert(
            &ident_string,
            (
                ident,
                definition,
                DefineSource::Source(self.file_stack.clone()),
            ),
        );
        Ok(())
    }

    pub(crate) fn directive_undef(
        &mut self,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
    ) -> Result<(), Error> {
        let ident = self.next_word(stream, None)?;
        if !ident.symbol().is_word() {
            return Err(ExpectedIdent::code(ident.as_ref().clone()));
        }
        let ident_string = ident.symbol().to_string();
        self.defines.remove(&ident_string);
        Self::expect_nothing_to_newline(stream)
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn directive_if(
        &mut self,
        pragma: &Pragma,
        command: Arc<Token>,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
    ) -> Result<(), Error> {
        fn read_value(
            stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
        ) -> Vec<Arc<Token>> {
            let mut tokens = Vec::new();
            while stream.peek().is_some() {
                let token = stream
                    .peek()
                    .expect("peeked in the if statement, so there should be a token")
                    .clone();
                if token.symbol().is_whitespace() || token.symbol().is_newline() {
                    break;
                }
                tokens.push(token);
                stream.next();
            }
            tokens
        }
        #[allow(clippy::type_complexity)]
        fn resolve_value(
            defines: &mut Defines,
            token: Arc<Token>,
        ) -> Result<(Arc<Vec<Arc<Token>>>, bool), Error> {
            if let Some((_, definition, _)) = defines.get_with_gen(&token, Some(token.position())) {
                if let Definition::Value(tokens) = definition {
                    return Ok((tokens, true));
                }
                return Err(IfUnitOrFunction::code(
                    token.as_ref().clone(),
                    &defines.clone(),
                ));
            }
            Ok((Arc::new(vec![token]), false))
        }
        self.skip_whitespace(stream, None);
        let left = read_value(stream);
        if !left.is_empty()
            && &Symbol::Word(String::from("__has_include"))
                == left
                    .first()
                    .expect("left is not empty, must exist")
                    .symbol()
        {
            if pragma.is_flagged(&Flag::Pe23IgnoreIfHasInclude) {
                debug!(
                    "ignoring __has_include due to pragma flag, this config will not be rapified"
                );
                self.no_rapify = true;
                self.ifstates.push_if(command, false);
                self.skip_to_after_newline(stream, None);
                return Ok(());
            }
            return Err(IfHasInclude::code(
                left.first()
                    .expect("left is not empty, must exist")
                    .as_ref()
                    .clone(),
            ));
        }
        let (left, left_defined) = if left.len() == 1 {
            resolve_value(
                &mut self.defines,
                left.into_iter()
                    .next()
                    .expect("length is 1, next will exist"),
            )?
        } else {
            (Arc::new(left), false)
        };
        if left.is_empty() {
            return Err(UnexpectedEOF::code(command.as_ref().clone()));
        }
        self.skip_whitespace(stream, None);
        #[allow(unused_assignments)]
        let mut operators = Vec::new();
        let (right, right_defined) = if stream.peek().map(|t| t.symbol()) == Some(&Symbol::Newline)
        {
            let pos = stream
                .peek()
                .expect("peeked in the if statement, so there should be a token")
                .position()
                .clone();
            if !left_defined {
                return Err(IfUndefined::code(left[0].as_ref().clone(), &self.defines));
            }
            let equals = Arc::new(Token::new(Symbol::Equals, pos.clone()));
            operators = vec![equals.clone(), equals];
            (
                Arc::new(vec![Arc::new(Token::new(Symbol::Digit(1), pos))]),
                false,
            )
        } else {
            operators = read_value(stream);
            self.skip_whitespace(stream, None);
            let right = read_value(stream);
            if right.is_empty() {
                return Err(UnexpectedEOF::code(
                    operators
                        .last()
                        .expect("right should exists at this point")
                        .as_ref()
                        .clone(),
                ));
            }
            if right.len() == 1 {
                resolve_value(
                    &mut self.defines,
                    right
                        .into_iter()
                        .next()
                        .expect("length is 1, next will exist"),
                )?
            } else {
                (Arc::new(right), false)
            }
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
                let Ok(left_f64) = left_string.parse::<f64>() else {
                    return Err(IfIncompatibleType::code(
                        &(left, left_defined),
                        operators,
                        &(right, right_defined),
                    ));
                };
                let Ok(right_f64) = right_string.parse::<f64>() else {
                    return Err(IfIncompatibleType::code(
                        &(left, left_defined),
                        operators,
                        &(right, right_defined),
                    ));
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
                return Err(IfInvalidOperator::code(
                    operators.iter().map(|t| t.as_ref().clone()).collect(),
                ));
            }
        };
        self.ifstates.push_if(command, read);
        Self::expect_nothing_to_newline(stream)?;
        Ok(())
    }

    pub(crate) fn directive_ifdef(
        &mut self,
        command: Arc<Token>,
        outcome: bool,
        stream: &mut PeekMoreIterator<impl Iterator<Item = Arc<Token>>>,
    ) -> Result<(), Error> {
        let ident = self.next_word(stream, None)?;
        if !ident.symbol().is_word() {
            return Err(ExpectedIdent::code(ident.as_ref().clone()));
        }
        let ident_string = ident.symbol().to_string();
        self.ifstates
            .push_if(command, self.defines.contains_key(&ident_string) == outcome);
        Self::expect_nothing_to_newline(stream)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use hemtt_workspace::reporting::{Definition, Symbol};

    use crate::processor::{Processor, pragma::Pragma, tests};

    #[test]
    fn directive_define_unit() {
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
    fn directive_define_value() {
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
