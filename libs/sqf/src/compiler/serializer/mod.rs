//! This module contains the minimum required for the serializing SQFC files.
//!
//! The entrypoint to this is [`Compiled`], however it contains raw lists of constants and variable names,
//! which can be cumbersome to manage. For this reason, you should use structs from the
//! [`compiler`][crate::compiler] module instead.

mod display;

pub use self::display::{DisplayConstant, DisplayInstructions};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};

use std::cmp::Ordering;
use std::io::{self, Read, Write};
use std::sync::Arc;

pub const VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SourceInfo {
    pub offset: u32,
    pub file_index: u8,
    pub file_line: u16,
}

impl SourceInfo {
    /// Serializes this source info to the given writer.
    ///
    /// # Errors
    /// [`std::io::Error`] if an error occurs while writing to the writer.
    pub fn serialize(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_u32::<LE>(self.offset)?;
        writer.write_u8(self.file_index)?;
        writer.write_u16::<LE>(self.file_line)?;
        Ok(())
    }

    pub(crate) fn deserialize(reader: &mut impl Read) -> io::Result<Self> {
        let offset = reader.read_u32::<LE>()?;
        let file_index = reader.read_u8()?;
        let file_line = reader.read_u16::<LE>()?;
        Ok(Self {
            offset,
            file_index,
            file_line,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodePointer {
    /// Points to a constant containing the source code for these instructions.
    Constant(u64),
    /// Points to an offset in the main code string
    Source {
        offset: u32,
        /// Actually u31, a bit is used as a flag
        length: u32,
    },
}

impl CodePointer {
    /// Serializes this code pointer to the given writer.
    ///
    /// # Errors
    /// [`std::io::Error`] if an error occurs while writing to the writer.
    pub fn serialize(&self, writer: &mut impl Write) -> io::Result<()> {
        match self {
            Self::Constant(constant_index) => {
                writer.write_u64::<LE>(*constant_index & 0x7FFF_FFFF)?;
            }
            Self::Source { offset, length } => {
                writer.write_u32::<LE>(*offset)?;
                writer.write_u32::<LE>(*length | 0x8000_0000)?;
            }
        };

        Ok(())
    }

    pub(crate) fn deserialize(reader: &mut impl Read) -> io::Result<Self> {
        let first = reader.read_u32::<LE>()?;
        if first & 0x8000_0000 == 0 {
            Ok(Self::Constant(u64::from(first)))
        } else {
            Ok(Self::Source {
                offset: first & 0x7FFF_FFFF,
                length: reader.read_u32::<LE>()?,
            })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    /// Counter to its name, this instruction seems to be used to *begin* statements.
    EndStatement,
    /// This instruction seems to take an index to the constant table, and pushes that constant onto the stack.
    Push(u16),
    /// This instruction seems to take an index to the name table,
    /// calling that command with that last value on the stack, placing result back on the stack.
    CallUnary(u16, SourceInfo),
    /// This instruction seems to take an index to the name table,
    /// calling that command with the last two values on the stack, placing the result back on the stack.
    CallBinary(u16, SourceInfo),
    /// This instruction seems to take an index to the name table,
    /// calling that command with no values from the stack, putting its result.
    CallNular(u16, SourceInfo),
    /// This instruction seems to take an index to the name table,
    /// assigning the last value on the stack to a variable with that name.
    AssignTo(u16, SourceInfo),
    /// This instruction seems to take an index to the name table,
    /// assigning the last value on the stack to a variable with that name, locally.
    AssignToLocal(u16, SourceInfo),
    /// This instruction seems to take an index to the name table,
    /// retrieving the value of that variable, placing it on the stack.
    GetVariable(u16, SourceInfo),
    /// This instruction seems to take a numeric length, which determines how many values
    /// from the stack it should consume to create an array, which it places back on the stack.
    MakeArray(u16, SourceInfo),
}

impl Instruction {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::EndStatement => "EndStatement",
            Self::Push(..) => "Push",
            Self::CallUnary(..) => "CallUnary",
            Self::CallBinary(..) => "CallBinary",
            Self::CallNular(..) => "CallNular",
            Self::AssignTo(..) => "AssignTo",
            Self::AssignToLocal(..) => "AssignToLocal",
            Self::GetVariable(..) => "GetVariable",
            Self::MakeArray(..) => "MakeArray",
        }
    }

    const fn to_byte(self) -> u8 {
        match self {
            Self::EndStatement => 0,
            Self::Push(..) => 1,
            Self::CallUnary(..) => 2,
            Self::CallBinary(..) => 3,
            Self::CallNular(..) => 4,
            Self::AssignTo(..) => 5,
            Self::AssignToLocal(..) => 6,
            Self::GetVariable(..) => 7,
            Self::MakeArray(..) => 8,
        }
    }

    /// Serializes this instruction to the given writer.
    ///
    /// # Errors
    /// [`std::io::Error`] if an error occurs while writing to the writer.
    /// [`SerializeError`] if the instruction contains an invalid index.
    pub fn serialize(self, compiled: &Compiled, writer: &mut impl Write) -> SerializeResult {
        writer.write_u8(self.to_byte())?;

        match self {
            Self::EndStatement => (),
            Self::Push(constant_index) => {
                compiled.assert_has_constant(constant_index)?;
                writer.write_u16::<LE>(constant_index)?;
            }
            Self::CallUnary(name_index, source_info)
            | Self::CallBinary(name_index, source_info)
            | Self::CallNular(name_index, source_info)
            | Self::AssignTo(name_index, source_info)
            | Self::AssignToLocal(name_index, source_info)
            | Self::GetVariable(name_index, source_info) => {
                source_info.serialize(writer)?;
                compiled.assert_has_name(name_index)?;
                writer.write_u16::<LE>(name_index)?;
            }
            Self::MakeArray(array_len, source_info) => {
                source_info.serialize(writer)?;
                writer.write_u16::<LE>(array_len)?;
            }
        };

        Ok(())
    }

    pub(crate) fn deserialize(reader: &mut impl Read) -> DeserializeResult<Self> {
        let tag = reader.read_u8()?;

        // EndStatement (0) and Push (1)
        let source_info = if matches!(tag, 0 | 1) {
            SourceInfo::default()
        } else {
            SourceInfo::deserialize(reader)?
        };

        match tag {
            0 => Ok(Self::EndStatement),
            1 => Ok(Self::Push(reader.read_u16::<LE>()?)),
            2 => Ok(Self::CallUnary(reader.read_u16::<LE>()?, source_info)),
            3 => Ok(Self::CallBinary(reader.read_u16::<LE>()?, source_info)),
            4 => Ok(Self::CallNular(reader.read_u16::<LE>()?, source_info)),
            5 => Ok(Self::AssignTo(reader.read_u16::<LE>()?, source_info)),
            6 => Ok(Self::AssignToLocal(reader.read_u16::<LE>()?, source_info)),
            7 => Ok(Self::GetVariable(reader.read_u16::<LE>()?, source_info)),
            8 => Ok(Self::MakeArray(reader.read_u16::<LE>()?, source_info)),
            tag => Err(DeserializeError::InvalidTagInstruction(tag)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instructions {
    pub contents: Vec<Instruction>,
    /// Points to a constant containing the source code for these instructions.
    pub source_pointer: CodePointer,
}

impl Instructions {
    #[must_use]
    pub const fn display<'a>(&'a self, compiled: &'a Compiled) -> DisplayInstructions<'a> {
        DisplayInstructions {
            compiled,
            instructions: self,
            indent: 0,
        }
    }

    /// Serializes these instructions to the given writer.
    ///
    /// # Errors
    /// [`std::io::Error`] if an error occurs while writing to the writer.
    /// [`SerializeError`] if the instructions contain an invalid index.
    pub fn serialize(&self, compiled: &Compiled, writer: &mut impl Write) -> SerializeResult {
        self.source_pointer.serialize(writer)?;
        let instructions_len =
            try_truncate_or(self.contents.len(), SerializeError::InstructionsLimit)?;
        writer.write_u32::<LE>(instructions_len)?;
        for instruction in &self.contents {
            instruction.serialize(compiled, writer)?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn deserialize(reader: &mut impl Read) -> DeserializeResult<Self> {
        let source_string_index = CodePointer::deserialize(reader)?;
        let instructions_len = reader.read_u32::<LE>()? as usize;
        let instructions = (0..instructions_len)
            .map(|_| Instruction::deserialize(reader))
            .collect::<DeserializeResult<Vec<Instruction>>>()?;
        Ok(Self {
            contents: instructions,
            source_pointer: source_string_index,
        })
    }
}

#[derive(Debug, Clone)]
pub enum Constant {
    Code(Instructions),
    String(Arc<str>),
    Scalar(f32),
    Boolean(bool),
    Array(Vec<Self>),
    ConsumeableArray(Vec<Self>),
    NularCommand(Arc<str>),
}

impl Constant {
    #[must_use]
    pub const fn display<'a>(&'a self, compiled: &'a Compiled) -> DisplayConstant<'a> {
        DisplayConstant {
            compiled,
            constant: self,
            indent: 0,
        }
    }

    const fn to_byte(&self) -> u8 {
        match *self {
            Self::Code(..) => 0,
            Self::String(..) => 1,
            Self::Scalar(..) => 2,
            Self::Boolean(..) => 3,
            Self::Array(..) | Self::ConsumeableArray(..) => 4,
            Self::NularCommand(..) => 5,
        }
    }

    /// Serializes this constant to the given writer.
    ///
    /// # Errors
    /// [`std::io::Error`] if an error occurs while writing to the writer.
    /// [`SerializeError`] if the constant contains an invalid index.
    pub fn serialize(&self, compiled: &Compiled, writer: &mut impl Write) -> SerializeResult {
        writer.write_u8(self.to_byte())?;

        match *self {
            Self::Code(ref instructions) => {
                instructions.serialize(compiled, writer)?;
            }
            Self::String(ref string) => {
                serialize_string(string, writer)?;
            }
            Self::Scalar(value) => {
                writer.write_f32::<LE>(value)?;
            }
            Self::Boolean(value) => {
                writer.write_u8(u8::from(value))?;
            }
            Self::Array(ref array) | Self::ConsumeableArray(ref array) => {
                let array_len = try_truncate_or(array.len(), SerializeError::ArrayTooLong)?;
                writer.write_u32::<LE>(array_len)?;
                for constant in array {
                    constant.serialize(compiled, writer)?;
                }
            }
            Self::NularCommand(ref command) => {
                serialize_string(command, writer)?;
            }
        };

        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn deserialize(reader: &mut impl Read) -> DeserializeResult<Self> {
        Ok(match reader.read_u8()? {
            0 => Self::Code(Instructions::deserialize(reader)?),
            1 => Self::String(deserialize_string(reader)?),
            2 => Self::Scalar(reader.read_f32::<LE>()?),
            3 => Self::Boolean(reader.read_u8()? == u8::from(true)),
            4 => Self::Array(deserialize_constant_array(reader)?),
            5 => Self::NularCommand(deserialize_string(reader)?),
            tag => return Err(DeserializeError::InvalidTagConstant(tag)),
        })
    }
}

impl PartialEq for Constant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Code(code1), Self::Code(code2)) => code1 == code2,
            (Self::String(string1), Self::String(string2)) => string1 == string2,
            (Self::Scalar(scalar1), Self::Scalar(scalar2)) => {
                f32::total_cmp(scalar1, scalar2) == Ordering::Equal
            }
            (Self::Boolean(boolean1), Self::Boolean(boolean2)) => boolean1 == boolean2,
            (Self::Array(array1), Self::Array(array2)) => array1 == array2,
            (Self::NularCommand(cmd1), Self::NularCommand(cmd2)) => {
                str::eq_ignore_ascii_case(cmd1, cmd2)
            }
            _ => false,
        }
    }
}

impl Eq for Constant {}

#[allow(dead_code)]
fn deserialize_constant_array(reader: &mut impl Read) -> DeserializeResult<Vec<Constant>> {
    let array_len = reader.read_u32::<LE>()? as usize;
    (0..array_len)
        .map(|_| Constant::deserialize(reader))
        .collect()
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Constants = 0,
    ConstantsCompressed = 1,
    LocationInfo = 2,
    Code = 3,
    CodeDebug = 4,
    NameCache = 5,
}

impl BlockType {
    const fn from_byte(tag: u8) -> Result<Self, u8> {
        match tag {
            0 => Ok(Self::Constants),
            1 => Ok(Self::ConstantsCompressed),
            2 => Ok(Self::LocationInfo),
            3 => Ok(Self::Code),
            4 => Ok(Self::CodeDebug),
            5 => Ok(Self::NameCache),
            tag => Err(tag),
        }
    }

    #[allow(dead_code)]
    fn deserialize(reader: &mut impl Read) -> DeserializeResult<Self> {
        Self::from_byte(reader.read_u8()?).map_err(DeserializeError::IncorrectBlockTypeTag)
    }
}

/// Contains all of the relevant data to serialize an SQFC script.
#[derive(Debug, Clone)]
pub struct Compiled {
    /// Also known as `codeIndex`, the index to a code constant that should be run when this script is called.
    pub entry_point: u16,
    /// Whether or not the constants cache should be compressed.
    pub constants_cache_compression: bool,
    /// Also known as `constants`, a list of hard-coded constant values used in the script.
    pub constants_cache: Vec<Constant>,
    /// Also knows as `commandNameDirectory`, a list of commands and variable names used in the script.
    pub names_cache: Vec<Arc<str>>,
    /// A list of file names which can be referenced by instructions for debug purposes.
    pub file_names: Vec<Arc<str>>,
}

impl Compiled {
    #[must_use]
    /// Returns a [`DisplayInstructions`] for the entrypoint of this compiled script.
    ///
    /// # Panics
    /// Panics if the entrypoint is invalid.
    pub fn display(&self) -> DisplayInstructions<'_> {
        self.get_entry_point()
            .expect("expected valid entrypoint")
            .display(self)
    }

    fn assert_has_constant(&self, index: u16) -> SerializeResult<()> {
        if self.constants_cache.len() <= index as usize {
            Err(SerializeError::InvalidConstantIndex(index))
        } else {
            Ok(())
        }
    }

    fn assert_has_name(&self, index: u16) -> SerializeResult<()> {
        if self.names_cache.len() <= index as usize {
            Err(SerializeError::InvalidNameIndex(index))
        } else {
            Ok(())
        }
    }

    #[must_use]
    pub fn get_entry_point(&self) -> Option<&Instructions> {
        match self.get_constant(self.entry_point)? {
            Constant::Code(entry_point) => Some(entry_point),
            _ => None,
        }
    }

    #[must_use]
    pub fn get_constant(&self, index: u16) -> Option<&Constant> {
        self.constants_cache.get(index as usize)
    }

    #[must_use]
    pub fn get_name(&self, index: u16) -> Option<&Arc<str>> {
        self.names_cache.get(index as usize)
    }

    /// Serializes this compiled script to the given writer.
    ///
    /// # Errors
    /// [`std::io::Error`] if an error occurs while writing to the writer.
    /// [`SerializeError`] if the script contains an invalid index.
    pub fn serialize(&self, writer: &mut impl Write) -> SerializeResult {
        // Version info
        writer.write_u32::<LE>(VERSION)?;

        // Command name directory
        writer.write_u8(BlockType::NameCache as u8)?;
        serialize_compress(writer, |buffer| {
            let names_cache_len =
                try_truncate_or(self.names_cache.len(), SerializeError::ListTooLongBlock)?;
            buffer.write_u16::<LE>(names_cache_len)?;
            for name in &self.names_cache {
                serialize_string(name, buffer)?;
            }

            Ok(())
        })?;

        if self.constants_cache_compression {
            // Constants compressed
            writer.write_u8(BlockType::ConstantsCompressed as u8)?;
            serialize_compress(writer, |buffer| self.serialize_constants_cache(buffer))?;
        } else {
            // Constants
            writer.write_u8(BlockType::Constants as u8)?;
            self.serialize_constants_cache(writer)?;
        };

        // Location info
        writer.write_u8(BlockType::LocationInfo as u8)?;
        let file_names_len =
            try_truncate_or(self.file_names.len(), SerializeError::ListTooLongBlock)?;
        writer.write_u16::<LE>(file_names_len)?;
        for file_name in &self.file_names {
            serialize_string(file_name, writer)?;
        }

        // Code (Entrypoint)
        writer.write_u8(BlockType::Code as u8)?;
        writer.write_u64::<LE>(u64::from(self.entry_point))?;

        Ok(())
    }

    fn serialize_constants_cache(&self, writer: &mut impl Write) -> SerializeResult {
        let constants_cache_len =
            try_truncate_or(self.constants_cache.len(), SerializeError::ListTooLongBlock)?;
        writer.write_u16::<LE>(constants_cache_len)?;
        for constant in &self.constants_cache {
            constant.serialize(self, writer)?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn deserialize_name_cache(reader: &mut impl Read) -> DeserializeResult<Vec<Arc<str>>> {
        let name_cache_len = reader.read_u16::<LE>()? as usize;
        (0..name_cache_len)
            .map(|_| deserialize_string(reader))
            .collect()
    }

    #[allow(dead_code)]
    fn deserialize_constants_cache(reader: &mut impl Read) -> DeserializeResult<Vec<Constant>> {
        let constants_cache_len = reader.read_u16::<LE>()? as usize;
        (0..constants_cache_len)
            .map(|_| Constant::deserialize(reader))
            .collect()
    }

    // TODO: fix this and make it public
    // compiler deserialization/decompilation isn't public right now because there's no way
    // to make it work properly without an LZO decoding algorithm that can decode from a buffer/stream
    // without knowing where the buffer/stream ends
    #[allow(dead_code)]
    pub(crate) fn deserialize(
        reader: &mut impl Read,
        name_cache_buffer_len: usize,
        constants_cache_buffer_len: usize,
    ) -> DeserializeResult<Self> {
        let version = reader.read_u32::<LE>()?;
        if version != VERSION {
            return Err(DeserializeError::IncorrectVersion(version));
        };

        let mut names_cache = None;
        let mut constants_cache = None;
        let mut file_names = None;
        let mut entry_point = None;

        while constants_cache.is_none()
            || file_names.is_none()
            || entry_point.is_none()
            || names_cache.is_none()
        {
            match BlockType::deserialize(reader)? {
                BlockType::Constants if constants_cache.is_none() => {
                    constants_cache = Some(Self::deserialize_constants_cache(reader)?);
                }
                BlockType::ConstantsCompressed if constants_cache.is_none() => {
                    let buffer = decompress_buffer(reader, constants_cache_buffer_len)?;
                    constants_cache =
                        Some(Self::deserialize_constants_cache(&mut buffer.as_slice())?);
                }
                BlockType::LocationInfo if file_names.is_none() => {
                    let file_names_len = reader.read_u16::<LE>()? as usize;
                    file_names = Some({
                        (0..file_names_len)
                            .map(|_| deserialize_string(reader))
                            .collect::<DeserializeResult<Vec<Arc<str>>>>()?
                    });
                }
                BlockType::Code if entry_point.is_none() => {
                    entry_point = Some(reader.read_u64::<LE>()? as u16);
                }
                BlockType::NameCache if names_cache.is_none() => {
                    let buffer = decompress_buffer(reader, name_cache_buffer_len)?;
                    names_cache = Some(Self::deserialize_name_cache(&mut buffer.as_slice())?);
                }
                block => return Err(DeserializeError::UnexpectedBlock(block)),
            };
        }

        Ok(Self {
            entry_point: entry_point.expect("entry point not found"),
            constants_cache_compression: true,
            constants_cache: constants_cache.expect("constants cache not found"),
            names_cache: names_cache.expect("names cache not found"),
            file_names: file_names.expect("file names not found"),
        })
    }
}

fn decompress_buffer(reader: &mut impl Read, size: usize) -> DeserializeResult<Vec<u8>> {
    let buffer_size = reader.read_u32::<LE>()? as usize;
    if reader.read_u8()? != 2 {
        return Err(DeserializeError::InvalidCompressionMode);
    };

    let mut buffer_uncompressed = vec![0; size];
    reader.read_exact(&mut buffer_uncompressed)?;
    let mut buffer = Vec::with_capacity(buffer_size);
    hemtt_lzo::decompress_to_slice(&buffer_uncompressed, &mut buffer)?;
    Ok(buffer)
}

fn serialize_compress<F>(writer: &mut impl Write, f: F) -> SerializeResult
where
    F: FnOnce(&mut Vec<u8>) -> SerializeResult,
{
    let mut buffer = Vec::new();
    f(&mut buffer)?;
    serialize_compress_buffer(&buffer, writer)
}

fn serialize_compress_buffer(buffer: impl AsRef<[u8]>, writer: &mut impl Write) -> SerializeResult {
    let buffer = buffer.as_ref();
    // Uncompressed size
    let buffer_size = try_truncate_or(buffer.len(), SerializeError::BufferSizeLimit)?;
    let len: usize = hemtt_lzo::worst_compress(std::mem::size_of_val(buffer));
    let mut buffer_compressed = Vec::with_capacity(len);
    hemtt_lzo::compress(buffer, &mut buffer_compressed)?;
    writer.write_u32::<LE>(buffer_size)?;
    // Compression method, always 2
    writer.write_u8(2)?;
    // Compressed data
    writer.write_all(&buffer_compressed)?;
    Ok(())
}

fn serialize_string(string: &str, writer: &mut impl Write) -> SerializeResult {
    let string_len = string.len();
    if string_len >= usize::pow(2, 24) {
        return Err(SerializeError::StringTooLong(string_len));
    };

    writer.write_u24::<LE>(string_len as u32)?;
    writer.write_all(string.as_bytes())?;
    Ok(())
}

fn deserialize_string(reader: &mut impl Read) -> DeserializeResult<Arc<str>> {
    let buffer_len = reader.read_u24::<LE>()? as usize;
    let mut buffer = vec![0; buffer_len];
    reader.read_exact(&mut buffer)?;
    let string = String::from_utf8(buffer)?;
    Ok(string.into())
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum DeserializeError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("invalid compression mode")]
    InvalidCompressionMode,
    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("incorrect version {0}")]
    IncorrectVersion(u32),
    #[error("incorrect block type tag {0:x}")]
    IncorrectBlockTypeTag(u8),
    #[error("invalid tag {0:x} for instruction")]
    InvalidTagInstruction(u8),
    #[error("invalid tag {0:x} for constant")]
    InvalidTagConstant(u8),
    #[error("unexpected block type {0:?}")]
    UnexpectedBlock(BlockType),
    #[error("lzo error")]
    LzoError(#[from] hemtt_lzo::LzoError),
}

type DeserializeResult<T> = Result<T, DeserializeError>;

#[derive(Debug, thiserror::Error)]
pub enum SerializeError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("cannot serialize string longer than 2^24 bytes, found string of size {0}")]
    StringTooLong(usize),
    #[error("cannot serialize array longer than 2^32 bytes, found array of size {0}")]
    ArrayTooLong(usize),
    #[error(
        "cannot serialize instructions list longer than 2^32 instructions, found list of size {0}"
    )]
    InstructionsLimit(usize),
    #[error("cannot serialize buffer list longer than 2^32 bytes, found buffer of size {0}")]
    BufferSizeLimit(usize),
    #[error("cannot serialize list longer than 2^16 in block, found list of size {0}")]
    ListTooLongBlock(usize),
    #[error("invalid constant index {0}, not found in compiled context")]
    InvalidConstantIndex(u16),
    #[error("invalid command/name index {0}, not found in compiled context")]
    InvalidNameIndex(u16),
    #[error("Lzo error {0}")]
    LzoError(#[from] hemtt_lzo::LzoError),
}

type SerializeResult<T = ()> = Result<T, SerializeError>;

fn try_truncate_or<T>(value: usize, err: fn(usize) -> SerializeError) -> SerializeResult<T>
where
    T: TryFrom<usize>,
{
    T::try_from(value).map_err(|_| err(value))
}
