use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use std::io::{Error, Read, Write};

use hemtt_io::*;

#[derive(Debug, Clone)]
pub struct Header {
    pub filename: String,
    pub method: u32,
    pub original: u32,
    pub reserved: u32,
    pub timestamp: u32,
    pub size: u32,
}

impl Header {
    pub fn read<I: Read>(input: &mut I) -> Result<(Self, usize), Error> {
        let mut size = 4 * 5;
        let filename = input.read_cstring()?.replace("/", "\\");
        size += filename.as_bytes().len() + 1;
        trace!("reading header of size: {} bytes", size);
        Ok((
            Self {
                filename,
                method: input.read_u32::<LittleEndian>()?,
                original: input.read_u32::<LittleEndian>()?,
                reserved: input.read_u32::<LittleEndian>()?,
                timestamp: input.read_u32::<LittleEndian>()?,
                size: input.read_u32::<LittleEndian>()?,
            },
            size,
        ))
    }

    pub fn write<O: Write>(&self, output: &mut O) -> Result<(), Error> {
        trace!("writing header for `{}`", self.filename);
        output.write_cstring(&self.filename)?;
        output.write_u32::<LittleEndian>(self.method)?;
        output.write_u32::<LittleEndian>(self.original)?;
        output.write_u32::<LittleEndian>(self.reserved)?;
        output.write_u32::<LittleEndian>(self.timestamp)?;
        output.write_u32::<LittleEndian>(self.size)?;
        Ok(())
    }
}

#[test]
fn read() {
    use std::io::Cursor;
    let (header, _) = crate::header::Header::read(&mut Cursor::new(String::from(
        "images\\mission.jpg��*\\*Wi",
    )))
    .unwrap();
    assert_eq!(header.filename, "images\\mission.jpg");
    assert_eq!(header.size, 1_546_304_959);
    assert_eq!(header.timestamp, 4_022_190_063);
}
