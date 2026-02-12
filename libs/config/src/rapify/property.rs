use byteorder::ReadBytesExt;
use chumsky::span::{SimpleSpan, Spanned};
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
                Self::Entry { value, .. } => match value.inner {
                    Value::Str(ref s) => s.rapified_length(),
                    Value::Number(ref n) => n.rapified_length(),
                    Value::Expression(ref e) => e.rapified_length(),
                    Value::Array(ref a) => a.rapified_length(),
                    Value::UnexpectedArray(_) | Value::Invalid(_) => unreachable!(),
                },
                Self::Class(c) => match c.inner {
                    Class::Root { .. } => panic!("root should not be a property"),
                    Class::Local { .. } => 4,
                    Class::External { .. } => 0,
                },
                Self::Delete(_) => 0,
                Self::MissingSemicolon(_) | Self::ExtraSemicolons(_) => unreachable!(),
            }
    }
}

impl Property {
    /// Get the code of the property
    #[must_use]
    pub fn property_code(&self) -> Vec<u8> {
        match self {
            Self::Entry { value, .. } => match value.inner {
                Value::Str(ref s) => vec![1, s.rapified_code()],
                Value::Number(ref n) => vec![1, n.rapified_code()],
                Value::Expression(ref e) => vec![1, e.rapified_code()],
                Value::Array(ref a) => {
                    if a.expand {
                        vec![5, 1, 0, 0, 0]
                    } else {
                        vec![2]
                    }
                }
                Value::UnexpectedArray(_) | Value::Invalid(_) => unreachable!(),
            },
            Self::Class(c) => match c.inner {
                Class::Local { .. } | Class::Root { .. } => vec![0],
                Class::External { .. } => vec![3],
            },
            Self::Delete(_) => {
                vec![4]
            }
            Self::MissingSemicolon(_) | Self::ExtraSemicolons(_) => unreachable!(),
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
        ) -> Result<Spanned<Ident>, std::io::Error> {
            Ok(Spanned {
                inner: Ident::new(&input.read_cstring()?),
                span: SimpleSpan::default(),
            })
        }
        let code = input.read_u8()?;
        Ok(match code {
            0 => {
                let name = read_name(input)?;
                let class_offset = input.read_u32::<byteorder::LittleEndian>()? as usize;
                let seek_back = input.stream_position()? as usize;
                #[allow(clippy::cast_possible_wrap)]
                input.seek(std::io::SeekFrom::Start(class_offset as u64))?;
                let class = Self::Class(Spanned {
                    inner: Class::derapify(input, Some(name))?,
                    span: SimpleSpan::default(),
                });
                input.seek(std::io::SeekFrom::Start(seek_back as u64))?;
                class
            }
            1 => {
                let subcode = input.read_u8()?;
                Self::Entry {
                    name: read_name(input)?,
                    value: Spanned {
                        inner: Value::derapify(input, subcode)?,
                        span: SimpleSpan::default(),
                    },
                    expected_array: false,
                }
            }
            2 => Self::Entry {
                name: read_name(input)?,
                value: Spanned {
                    inner: Value::Array(Array::derapify(input, false)?),
                    span: SimpleSpan::default(),
                },
                expected_array: true,
            },
            3 => Self::Class(Spanned {
                inner: Class::External {
                    name: read_name(input)?,
                },
                span: SimpleSpan::default(),
            }),
            4 => Self::Delete(read_name(input)?),
            5 => {
                let expand = input.read_u8()? == 1;
                input.seek_relative(3)?;
                Self::Entry {
                    name: read_name(input)?,
                    value: Spanned {
                        inner: Value::Array(Array::derapify(input, expand)?),
                        span: SimpleSpan::default(),
                    },
                    expected_array: true,
                }
            }
            _ => unreachable!(),
        })
    }
}
