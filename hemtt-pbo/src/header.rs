use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use std::{
    io::{Error, Read, Write},
    ops::Deref,
};

use hemtt_io::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(u32);
impl Deref for Timestamp {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Timestamp {
    pub fn from_u32(t: u32) -> Self {
        Self(t)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Header {
    pub filename: String,
    pub method: u32,
    pub original: u32,
    pub reserved: u32,
    pub timestamp: Timestamp,
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
                timestamp: Timestamp(input.read_u32::<LittleEndian>()?),
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
        output.write_u32::<LittleEndian>(*self.timestamp)?;
        output.write_u32::<LittleEndian>(self.size)?;
        Ok(())
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }
    pub fn method(&self) -> u32 {
        self.method
    }
    pub fn original(&self) -> u32 {
        self.original
    }
    pub fn reserved(&self) -> u32 {
        self.reserved
    }
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
    pub fn size(&self) -> u32 {
        self.size
    }
}

#[test]
fn read() {
    use std::io::Cursor;

    // There is additonal junk at the end of this header for the reading test
    let bytes: Vec<u8> = vec![
        105, 109, 97, 103, 101, 115, 92, 109, 105, 115, 115, 105, 111, 110, 46, 106, 112, 103, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 191, 189, 239, 191, 189, 42, 92, 92, 42, 87, 8, 0,
        105,
    ];

    let (header, _) =
        crate::header::Header::read(&mut Cursor::new(String::from_utf8(bytes.clone()).unwrap()))
            .unwrap();
    assert_eq!(header.filename, "images\\mission.jpg");
    assert_eq!(header.method, 0);
    assert_eq!(header.original, 0);
    assert_eq!(header.reserved, 0);
    assert_eq!(header.timestamp, Timestamp(4_022_190_063));
    assert_eq!(*header.timestamp, 4_022_190_063);
    assert_eq!(header.size, 1_546_304_959);

    let mut write_buf = Vec::new();
    header.write(&mut Cursor::new(&mut write_buf)).unwrap();
    assert_eq!(
        String::from_utf8(write_buf).unwrap(),
        String::from_utf8(bytes[..39].to_vec()).unwrap()
    );
}
