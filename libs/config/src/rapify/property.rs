use byteorder::ReadBytesExt;
use hemtt_common::io::ReadExt;

use crate::{Array, Class, Ident, Property, Value};

use super::{Derapify, Rapify};

impl Rapify for Property {
    fn rapify<O: std::io::Write>(
        &self,
        _output: &mut O,
        _offset: usize,
    ) -> Result<usize, std::io::Error> {
        unreachable!()
    }

    fn rapified_length(&self) -> usize {
        self.property_code().len()
            + match self {
                Self::Entry { value, .. } => match value {
                    Value::Str(s) => s.rapified_length(),
                    Value::Number(n) => n.rapified_length(),
                    Value::Expression(e) => e.rapified_length(),
                    Value::Array(a) => a.rapified_length(),
                    Value::UnexpectedArray(_) | Value::Invalid(_) => unreachable!(),
                },
                Self::Class(c) => match c {
                    Class::Root { .. } => panic!("root should not be a property"),
                    Class::Local { .. } => 4,
                    Class::External { .. } => 0,
                },
                Self::Delete(_) => 0,
                Self::MissingSemicolon(_, _) => unreachable!(),
            }
    }
}

impl Property {
    /// Get the code of the property
    #[must_use]
    pub fn property_code(&self) -> Vec<u8> {
        match self {
            Self::Entry { value, .. } => match value {
                Value::Str(s) => vec![1, s.rapified_code()],
                Value::Number(n) => vec![1, n.rapified_code()],
                Value::Expression(e) => vec![1, e.rapified_code()],
                Value::Array(a) => {
                    if a.expand {
                        vec![5, 1, 0, 0, 0]
                    } else {
                        vec![2]
                    }
                }
                Value::UnexpectedArray(_) | Value::Invalid(_) => unreachable!(),
            },
            Self::Class(c) => match c {
                Class::Local { .. } | Class::Root { .. } => vec![0],
                Class::External { .. } => vec![3],
            },
            Self::Delete(_) => {
                vec![4]
            }
            Self::MissingSemicolon(_, _) => unreachable!(),
        }
    }

    #[must_use]
    pub fn derapify_offset(&self) -> usize {
        match self {
            Self::Class(c) => c.rapified_length(),
            _ => 0,
        }
    }
}

impl Derapify for Property {
    fn derapify<I: std::io::Read + std::io::Seek>(input: &mut I) -> Result<Self, std::io::Error> {
        fn read_name<I: std::io::Read + std::io::Seek>(
            input: &mut I,
        ) -> Result<Ident, std::io::Error> {
            let start = input.stream_position()? as usize;
            let name = input.read_cstring()?;
            let name = Ident::new(name, start..input.stream_position()? as usize);
            Ok(name)
        }
        let code = input.read_u8()?;
        Ok(match code {
            0 => {
                let name = read_name(input)?;
                let class_offset = input.read_u32::<byteorder::LittleEndian>()? as usize;
                let seek_back = input.stream_position()? as usize;
                #[allow(clippy::cast_possible_wrap)]
                input.seek(std::io::SeekFrom::Start(class_offset as u64))?;
                let class = Self::Class(Class::derapify(input, Some(name))?);
                input.seek(std::io::SeekFrom::Start(seek_back as u64))?;
                class
            }
            1 => {
                let subcode = input.read_u8()?;
                Self::Entry {
                    name: read_name(input)?,
                    value: Value::derapify(input, subcode)?,
                    expected_array: false,
                }
            }
            2 => Self::Entry {
                name: read_name(input)?,
                value: Value::Array(Array::derapify(input, false)?),
                expected_array: true,
            },
            3 => Self::Class(Class::External {
                name: read_name(input)?,
            }),
            4 => Self::Delete(read_name(input)?),
            5 => {
                let expand = input.read_u8()? == 1;
                input.seek_relative(3)?;
                Self::Entry {
                    name: read_name(input)?,
                    value: Value::Array(Array::derapify(input, expand)?),
                    expected_array: true,
                }
            }
            _ => unreachable!(),
        })
    }
}
