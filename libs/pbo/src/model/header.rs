use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use hemtt_io::ReadExt;

use crate::{ReadPbo, WritePbo};

use super::mime::Mime;

#[derive(Clone, Default, Debug)]
pub struct Header {
    filename: String,
    mime: Mime,
    original: u32,
    reserved: u32,
    timestamp: u32,
    size: u32,
}

impl Header {
    #[must_use]
    pub fn new_for_file(filename: String, size: u32) -> Self {
        Self {
            filename,
            original: size,
            size,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn ext() -> Self {
        Self {
            filename: String::from(""),
            mime: Mime::Vers,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn filename(&self) -> &str {
        &self.filename
    }

    #[must_use]
    pub const fn mime(&self) -> &Mime {
        &self.mime
    }

    #[must_use]
    pub const fn original(&self) -> u32 {
        self.original
    }

    #[must_use]
    pub const fn reserved(&self) -> u32 {
        self.reserved
    }

    #[must_use]
    pub const fn timestamp(&self) -> u32 {
        self.timestamp
    }

    #[must_use]
    pub const fn size(&self) -> u32 {
        self.size
    }
}

impl WritePbo for Header {
    fn write_pbo<O: std::io::Write>(&self, output: &mut O) -> Result<(), crate::error::Error> {
        output.write_all(self.filename.as_bytes())?;
        output.write_all(&[0])?;
        output.write_u32::<LittleEndian>(self.mime.as_u32())?;
        output.write_u32::<LittleEndian>(self.original)?;
        output.write_u32::<LittleEndian>(self.reserved)?;
        output.write_u32::<LittleEndian>(self.timestamp)?;
        output.write_u32::<LittleEndian>(self.size)?;
        Ok(())
    }
}

impl ReadPbo for Header {
    fn read_pbo<I: std::io::Read>(input: &mut I) -> Result<(Self, usize), crate::error::Error> {
        let mut size = 4 * 5;
        let filename = input.read_cstring()?;
        size += filename.len() + 1;
        let mime = input.read_u32::<LittleEndian>()?;
        Ok((
            Self {
                filename,
                mime: Mime::from_u32(mime).ok_or(crate::error::Error::UnsupportedMime(mime))?,
                original: input.read_u32::<LittleEndian>()?,
                reserved: input.read_u32::<LittleEndian>()?,
                timestamp: input.read_u32::<LittleEndian>()?,
                size: input.read_u32::<LittleEndian>()?,
            },
            size,
        ))
    }
}
