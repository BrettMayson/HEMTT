use std::io::Cursor;

use byteorder::{LittleEndian, WriteBytesExt};

use crate::{Class, Entry, Property};

use super::{compressed_int_len, Rapify, WriteExt};

impl Rapify for Class {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        offset: usize,
    ) -> Result<usize, std::io::Error> {
        let mut written = 0;

        match self {
            Self::External { name } => {
                output.write_all(&[3])?;
                output.write_cstring(&name.to_string())?;
                written += 1;
            }
            Self::Local {
                name: _,
                parent,
                children,
            } => {
                if let Some(parent) = &parent {
                    output.write_cstring(parent.to_string())?;
                    written += parent.to_string().len() + 1;
                } else {
                    written += output.write(b"\0")?;
                }
                written += output.write_compressed_int(children.0 .0.len() as u32)?;

                let children_len = children
                    .0
                     .0
                    .iter()
                    .map(|(n, p)| n.len() + 1 + p.rapified_length())
                    .sum::<usize>();
                let mut class_offset = offset + written + children_len;
                let mut class_bodies: Vec<Cursor<Box<[u8]>>> = Vec::new();
                let pre_children = written;

                for (name, property) in &children.0 .0 {
                    let pre_write = written;
                    let code = property.property_code();
                    output.write_all(&code)?;
                    written += code.len();
                    output.write_cstring(name)?;
                    written += name.len() + 1;
                    match property {
                        Property::Entry(e) => {
                            written += e.rapify(output, offset)?;
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
                    }
                    assert_eq!(
                        written - pre_write,
                        property.rapified_length() + name.len() + 1
                    );
                }

                assert_eq!(written - pre_children, children_len);

                for cursor in class_bodies {
                    output.write_all(cursor.get_ref())?;
                    written += cursor.get_ref().len();
                }
            }
        }

        Ok(written)
    }

    fn rapified_length(&self) -> usize {
        match self {
            Self::External { .. } => 0,
            Self::Local {
                name: _,
                parent,
                children,
            } => {
                let parent_length = parent.as_ref().map_or(0, |parent| parent.to_string().len());
                parent_length
                    + 1
                    + compressed_int_len(children.0 .0.len() as u32)
                    + children
                        .0
                         .0
                        .iter()
                        .map(|(n, p)| {
                            n.len()
                                + 1
                                + p.rapified_length()
                                + match p {
                                    Property::Class(c) => c.rapified_length(),
                                    _ => 0,
                                    // Property::Delete(i) => i.to_string().len() + 1,
                                }
                        })
                        .sum::<usize>()
            }
        }
    }
}

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
                Self::Entry(e) => {
                    match e {
                        Entry::Str(s) => s.rapified_length(),
                        Entry::Number(n) => n.rapified_length(),
                        Entry::Array(a) => a.rapified_length(),
                        // Entry::Array(a) => {
                        //     compressed_int_len(a.elements.len() as u32)
                        //         + a.elements
                        //             .iter()
                        //             .map(Rapify::rapified_length)
                        //             .sum::<usize>()
                        // }
                    }
                }
                Self::Class(c) => match c {
                    Class::Local { .. } => 4,
                    Class::External { .. } => 0,
                },
                Self::Delete(_) => 0,
            }
    }
}
