#![allow(clippy::cast_possible_truncation)]

//! Compiles a list of statements into an intermediate form that can be serialized.
//!
//! Since [`Compiled`]'s names and constants lists can be difficult to manage,
//! this module contains structs that allow for the creation of a sort of intermediate form
//! which can generate these lists automatically.
//!
//! The main entrypoint to this is the [`Statements`][crate::Statements] struct, which can be
//! converted to a serializable [`Compiled`] via [`Statements::compile`][crate::Statements].

pub mod optimizer;
pub mod serializer;

use std::{ops::Range, sync::Arc};

use hemtt_workspace::reporting::Processed;
use serializer::CodePointer;

use self::serializer::{Compiled, Constant, Instruction, Instructions, SourceInfo};
use crate::{Error, Expression, Statement, Statements};

impl Statements {
    /// Converts this statements list into a [`Compiled`].
    /// A file name must be provided for debugging purposes.
    ///
    /// # Errors
    /// [`CompileError`] is returned if the statements list contains an invalid name.
    pub fn compile(&self, processed: &Processed) -> CompileResult<Compiled> {
        let mut ctx = Context {
            constants_cache: Vec::new(),
            names_cache: Vec::new(),
        };
        let entrypoint_code = self.compile_to_instructions(processed, &mut ctx, true)?;
        let entrypoint_index = ctx.constants_cache.len() as u16;
        ctx.constants_cache.push(Constant::Code(entrypoint_code));
        Ok(Compiled {
            entry_point: entrypoint_index,
            constants_cache_compression: true,
            constants_cache: ctx.constants_cache,
            names_cache: ctx.names_cache,
            file_names: processed
                .sources()
                .iter()
                .map(|(s, _)| s.as_virtual_str().into())
                .collect(),
        })
    }

    /// Compiles this statements list to a writer.
    ///
    /// # Errors
    /// [`Error`] is returned if the statements list contains an invalid name.
    pub fn compile_to_writer(
        &self,
        processed: &Processed,
        mut writer: impl std::io::Write,
    ) -> Result<(), Error> {
        Ok(self.compile(processed)?.serialize(&mut writer)?)
    }

    pub(crate) fn compile_to_instructions(
        &self,
        processed: &Processed,
        ctx: &mut Context,
        is_root: bool,
    ) -> CompileResult<Instructions> {
        let mut instructions = Vec::new();
        for statement in &self.content {
            statement.compile_instructions(&mut instructions, processed, ctx)?;
        }

        let source_pointer = if is_root {
            CodePointer::Constant(u64::from(
                ctx.add_constant(Constant::String(processed.clean_output().into()))?,
            ))
        } else {
            let offset = processed.clean_span(self.span.clone());
            let length = if self.content.is_empty() {
                0
            } else {
                offset.len() as u32
            };
            CodePointer::Source {
                offset: offset.start as u32,
                length,
            }
        };
        Ok(Instructions {
            contents: instructions,
            source_pointer,
        })
    }
}

#[must_use]
/// Converts a location in the processed file to a [`SourceInfo`].
///
/// # Panics
/// Panics if the location is not mapped.
pub fn location_to_source(processed: &Processed, location: &Range<usize>) -> SourceInfo {
    let map = processed.mapping(location.start).expect(
        "location not in mapping, this should not happen as the location is from the processed file",
    ).original();
    let offset = processed.get_byte_offset(location.start);
    SourceInfo {
        offset,
        file_index: processed
            .sources()
            .iter()
            .position(|(p, _)| p == map.path())
            .expect("file not in sources")
            .try_into()
            .expect("file index too large"),
        file_line: map.start().line() as u16,
    }
}

impl Statement {
    pub(crate) fn compile_instructions(
        &self,
        instructions: &mut Vec<Instruction>,
        processed: &Processed,
        ctx: &mut Context,
    ) -> CompileResult {
        instructions.push(Instruction::EndStatement);
        match *self {
            Self::AssignGlobal(ref name, ref expression, ref location) => {
                expression.compile_instructions(instructions, processed, ctx)?;
                let name_index = ctx.add_name(name)?;
                instructions.push(Instruction::AssignTo(
                    name_index,
                    location_to_source(processed, location),
                ));
            }
            Self::AssignLocal(ref name, ref expression, ref location) => {
                expression.compile_instructions(instructions, processed, ctx)?;
                let name_index = ctx.add_name(name)?;
                instructions.push(Instruction::AssignToLocal(
                    name_index,
                    location_to_source(processed, location),
                ));
            }
            Self::Expression(ref expression, _) => {
                expression.compile_instructions(instructions, processed, ctx)?;
            }
        };

        Ok(())
    }
}

impl Expression {
    pub(crate) fn compile_instructions(
        &self,
        instructions: &mut Vec<Instruction>,
        processed: &Processed,
        ctx: &mut Context,
    ) -> CompileResult {
        match self.compile_constant(processed, ctx)? {
            Some(constant) => {
                fn push_constant(
                    constant: Constant,
                    instructions: &mut Vec<Instruction>,
                    ctx: &mut Context,
                ) -> CompileResult {
                    if let Constant::Array(items) = constant {
                        let len = items.len();
                        for item in items {
                            push_constant(item, instructions, ctx)?;
                        }
                        instructions.push(Instruction::MakeArray(
                            len.try_into().expect("array too long"),
                            SourceInfo {
                                offset: 0,
                                file_index: 0,
                                file_line: 0,
                            },
                        ));
                    } else if let Constant::ConsumeableArray(_) = &constant {
                        // Only safe because we know this array will be consumed on use and won't be modifieable
                        instructions.push(Instruction::Push(ctx.add_constant(constant)?));
                    } else {
                        instructions.push(Instruction::Push(ctx.add_constant(constant)?));
                    }
                    Ok(())
                }
                push_constant(constant, instructions, ctx)?;
            }
            None => match *self {
                Self::ConsumeableArray(..) => {
                    unreachable!("couldn't make ConsumeableArray a const");
                }
                Self::Array(ref array, ref location) => {
                    let array_len = array
                        .len()
                        .try_into()
                        .map_err(|_| CompileError::ListTooLong)?;
                    for array_expr in array {
                        array_expr.compile_instructions(instructions, processed, ctx)?;
                    }

                    instructions.push(Instruction::MakeArray(
                        array_len,
                        location_to_source(processed, location),
                    ));
                }
                Self::NularCommand(ref command, ref location) => {
                    let name_index = ctx.add_name(command.as_str())?;
                    instructions.push(Instruction::CallNular(
                        name_index,
                        location_to_source(processed, location),
                    ));
                }
                Self::UnaryCommand(ref command, ref expr, ref location) => {
                    expr.compile_instructions(instructions, processed, ctx)?;
                    let name_index = ctx.add_name(command.as_str())?;
                    instructions.push(Instruction::CallUnary(
                        name_index,
                        location_to_source(processed, location),
                    ));
                }
                Self::BinaryCommand(ref command, ref expr1, ref expr2, ref location) => {
                    expr1.compile_instructions(instructions, processed, ctx)?;
                    expr2.compile_instructions(instructions, processed, ctx)?;
                    let name_index = ctx.add_name(command.as_str())?;
                    instructions.push(Instruction::CallBinary(
                        name_index,
                        location_to_source(processed, location),
                    ));
                }
                Self::Variable(ref name, ref location) => {
                    let name_index = ctx.add_name(name)?;
                    instructions.push(Instruction::GetVariable(
                        name_index,
                        location_to_source(processed, location),
                    ));
                }
                Self::Code(_)
                | Self::String(_, _, _)
                | Self::Number(_, _)
                | Self::Boolean(_, _) => {
                    unreachable!("constant should have been handled")
                }
            },
        };

        Ok(())
    }

    pub(crate) fn compile_constant(
        &self,
        processed: &Processed,
        ctx: &mut Context,
    ) -> CompileResult<Option<Constant>> {
        Ok(match *self {
            Self::Code(ref statements) => Some(Constant::Code(
                statements.compile_to_instructions(processed, ctx, false)?,
            )),
            Self::String(ref string, _, _) => Some(Constant::String(string.clone())),
            Self::Number(crate::Scalar(number), _) => Some(Constant::Scalar(number)),
            Self::Boolean(boolean, _) => Some(Constant::Boolean(boolean)),
            Self::Array(ref array, ..) => array
                .iter()
                .map(|value| value.clone().compile_constant(processed, ctx))
                .collect::<CompileResult<Option<Vec<Constant>>>>()?
                .map(Constant::Array),
            Self::ConsumeableArray(ref array, ..) => array
                .iter()
                .map(|value| value.clone().compile_constant(processed, ctx))
                .collect::<CompileResult<Option<Vec<Constant>>>>()?
                .map(Constant::ConsumeableArray),
            Self::NularCommand(ref command, ..) if command.is_constant() => {
                let command = try_normalize_name(&command.name)?;
                debug_assert_ne!(
                    &*command, "true",
                    "do not provide `true` as a nular constant command"
                );
                debug_assert_ne!(
                    &*command, "false",
                    "do not provide `false` as a nular constant command"
                );
                Some(Constant::NularCommand(command))
            }
            _ => None,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error("cannot convert list longer than 2^16 elements")]
    ListTooLong,
    #[error("invalid name {0}")]
    InvalidName(String),
}

type CompileResult<T = ()> = Result<T, CompileError>;

#[derive(Debug)]
pub(crate) struct Context {
    constants_cache: Vec<Constant>,
    names_cache: Vec<Arc<str>>,
}

impl Context {
    pub(crate) fn add_constant(&mut self, constant: Constant) -> CompileResult<u16> {
        add_or_get_index(&mut self.constants_cache, constant)
    }

    pub(crate) fn add_name(&mut self, name: &str) -> CompileResult<u16> {
        add_or_get_index(&mut self.names_cache, try_normalize_name(name)?)
    }
}

fn try_normalize_name(name: &str) -> CompileResult<Arc<str>> {
    let name_lower = name.to_ascii_lowercase();
    if crate::parser::database::is_valid_command(&name_lower) {
        Ok(name_lower.into())
    } else {
        Err(CompileError::InvalidName(name.to_owned()))
    }
}

fn add_or_get_index<T: PartialEq>(collection: &mut Vec<T>, value: T) -> CompileResult<u16> {
    collection
        .iter()
        .position(|item| item == &value)
        .unwrap_or_else(|| {
            let value_index = collection.len();
            collection.push(value);
            value_index
        })
        .try_into()
        .map_err(|_| CompileError::ListTooLong)
}
