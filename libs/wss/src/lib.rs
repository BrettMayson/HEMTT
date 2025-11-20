use std::io::{BufReader, Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

mod compression;
mod error;
mod mp3;
mod ogg;
mod wav;

pub use compression::Compression;
pub use error::Error;

pub struct Wss {
    compression: Compression,
    format: u16,
    channels: u16,
    sample_rate: u32,
    bytes_per_second: u32,
    block_align: u16,
    bits_per_sample: u16,
    output_size: u16,
    channel_data: Vec<Vec<i16>>,
}

impl Wss {
    /// Read a WSS file from the input
    ///
    /// # Errors
    /// [`std::io::Error`] if an IO error occurs
    /// [`Error::UnsupportedFileType`] if the file is not a WSS file
    /// [`Error::InvalidCompressionValue`] if the compression value is invalid
    pub fn read<R: Read>(input: R) -> Result<Self, Error> {
        let mut reader = BufReader::new(input);

        let mut buffer = [0; 4];
        reader.read_exact(&mut buffer)?;
        if &buffer != b"WSS0" {
            return Err(Error::UnsupportedFileType(
                String::from_utf8_lossy(&buffer).to_string(),
            ));
        }

        let compression = reader.read_u32::<LittleEndian>()?;
        let format = reader.read_u16::<LittleEndian>()?;
        let channels = reader.read_u16::<LittleEndian>()?;
        let sample_rate = reader.read_u32::<LittleEndian>()?;
        let bytes_per_second = reader.read_u32::<LittleEndian>()?;
        let block_align = reader.read_u16::<LittleEndian>()?;
        let bits_per_sample = reader.read_u16::<LittleEndian>()?;
        let output_size = reader.read_u16::<LittleEndian>()?;

        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        let compression = Compression::from_u32(compression)?;

        Ok(Self {
            compression,
            format,
            channels,
            sample_rate,
            bytes_per_second,
            block_align,
            bits_per_sample,
            output_size,
            channel_data: compression.decompress(&data, channels),
        })
    }

    /// Write the WSS file to the output
    ///
    /// # Errors
    /// [`std::io::Error`] if an IO error occurs
    pub fn write<O: Write>(&self, output: &mut O) -> Result<(), Error> {
        output.write_all(b"WSS0")?;
        output.write_u32::<LittleEndian>(self.compression.to_u32())?;
        output.write_u16::<LittleEndian>(self.format)?;
        output.write_u16::<LittleEndian>(self.channels)?;
        output.write_u32::<LittleEndian>(self.sample_rate)?;
        output.write_u32::<LittleEndian>(self.bytes_per_second)?;
        output.write_u16::<LittleEndian>(self.block_align)?;
        output.write_u16::<LittleEndian>(self.bits_per_sample)?;
        output.write_u16::<LittleEndian>(self.output_size)?;
        output.write_all(&self.compression.compress(&self.channel_data))?;

        Ok(())
    }

    pub const fn set_compression(&mut self, compression: Compression) {
        self.compression = compression;
    }

    #[must_use]
    pub const fn format(&self) -> u16 {
        self.format
    }

    #[must_use]
    pub const fn channels(&self) -> u16 {
        self.channels
    }

    #[must_use]
    pub const fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    #[must_use]
    pub const fn bytes_per_second(&self) -> u32 {
        self.bytes_per_second
    }

    #[must_use]
    pub const fn block_align(&self) -> u16 {
        self.block_align
    }

    #[must_use]
    pub const fn bits_per_sample(&self) -> u16 {
        self.bits_per_sample
    }

    #[must_use]
    pub const fn output_size(&self) -> u16 {
        self.output_size
    }

    #[must_use]
    pub const fn compression(&self) -> &Compression {
        &self.compression
    }

    #[must_use]
    pub fn size(&self) -> usize {
        self.channel_data.iter().map(std::vec::Vec::len).sum()
    }
}
