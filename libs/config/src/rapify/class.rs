use std::io::Cursor;

use byteorder::{LittleEndian, WriteBytesExt};
use hemtt_common::io::{ReadExt, WriteExt, compressed_int_len};

use crate::{Class, Ident, Property};

use super::{Derapify, Rapify};

impl Rapify for Class {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        offset: usize,
    ) -> Result<usize, std::io::Error> {
        let mut written = 0;
        match self {
            Self::Local { properties, .. } | Self::Root { properties } => {
                let parent = self.parent();
                if let Some(parent) = &parent {
                    output.write_cstring(parent.as_str())?;
                    written += parent.as_str().len() + 1;
                } else {
                    written += output.write(b"\0")?;
                }
                written += output.write_compressed_int(properties.len() as u32)?;

                let properties_len = properties
                    .iter()
                    .map(|p| p.name().len() + 1 + p.rapified_length())
                    .sum::<usize>();
                let mut class_offset = offset + written + properties_len + 4;
                let mut class_bodies: Vec<Cursor<Box<[u8]>>> = Vec::new();
                let pre_properties = written;

                for property in properties {
                    let pre_write = written;
                    let code = property.property_code();
                    output.write_all(&code)?;
                    written += code.len();
                    output.write_cstring(property.name().as_str())?;
                    written += property.name().len() + 1;
                    match property {
                        Property::Entry { value, .. } => {
                            written += value.rapify(output, offset)?;
                        }
                        Property::Class(c) => {
                            if let Self::Local { .. } = c {
                                output.write_u32::<LittleEndian>(class_offset as u32)?;
                                written += 4;
                                let buffer: Box<[u8]> =
                                    vec![0; c.rapified_length()].into_boxed_slice();
                                let mut cursor = Cursor::new(buffer);
                                let body_size = c.rapify(&mut cursor, class_offset)?;
                                assert_eq!(body_size, c.rapified_length());
                                class_offset += body_size;
                                class_bodies.push(cursor);
                            }
                        }
                        Property::Delete(_) => continue,
                        Property::MissingSemicolon(_, _) => unreachable!(),
                    }
                    assert_eq!(
                        written - pre_write,
                        property.rapified_length() + property.name().len() + 1
                    );
                }

                assert_eq!(written - pre_properties, properties_len);

                output.write_u32::<LittleEndian>(class_offset as u32)?;
                written += 4;

                for cursor in class_bodies {
                    output.write_all(cursor.get_ref())?;
                    written += cursor.get_ref().len();
                }
            }
            Self::External { name } => {
                output.write_all(&[3])?;
                output.write_cstring(name.as_str())?;
                written += 1;
            }
        }
        Ok(written)
    }

    fn rapified_length(&self) -> usize {
        match self {
            Self::External { .. } => 0,
            Self::Local { properties, .. } | Self::Root { properties, .. } => {
                let parent_length = self.parent().map_or(0, Ident::len);
                parent_length
                    + 1 // parent null terminator
                    + 4 // offset to next class
                    + compressed_int_len(properties.len() as u32)
                    + properties
                        .iter()
                        .map(|p| {
                            p.name().len()
                                + 1 // name null terminator
                                + p.rapified_length()
                                + match p {
                                    Property::Class(c) => c.rapified_length(),
                                    _ => 0,
                                }
                        })
                        .sum::<usize>()
            }
        }
    }
}

impl Class {
    /// Derapifies a class from the input stream.
    ///
    /// # Errors
    /// [`std::io::Error`] if the input stream is invalid or if the class cannot be read.
    pub fn derapify<I: std::io::Read + std::io::Seek>(
        input: &mut I,
        name: Option<Ident>,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let start = input.stream_position()? as usize;
        let parent = input.read_cstring()?;
        let parent = if parent.is_empty() {
            None
        } else {
            Some(Ident::new(parent, start..input.stream_position()? as usize))
        };
        let properties_len = input.read_compressed_int()?;
        let mut properties = Vec::with_capacity(properties_len as usize);
        let mut end_offset = 0;
        for _ in 0..properties_len {
            let prop = Property::derapify(input)?;
            end_offset += prop.derapify_offset();
            properties.push(prop);
        }
        #[allow(clippy::cast_possible_wrap)]
        input.seek_relative(end_offset as i64)?;
        Ok(if let Some(name) = name {
            Self::Local {
                name,
                parent,
                properties,
                err_missing_braces: false,
            }
        } else {
            Self::Root { properties }
        })
    }
}
