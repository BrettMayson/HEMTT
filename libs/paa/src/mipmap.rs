use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};
use texpresso::Format;

#[derive(Debug)]
pub struct MipMap {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
    format: Format,
}

impl MipMap {
    /// Read the `MipMap` from the given input
    ///
    /// # Errors
    /// [`std::io::Error`] if the input is not readable, or the `MipMap` is invalid
    pub fn from_stream<I: Read>(format: Format, stream: &mut I) -> Result<Self, std::io::Error> {
        let width = stream.read_u16::<LittleEndian>()?;
        let height = stream.read_u16::<LittleEndian>()?;
        let length = stream.read_u24::<LittleEndian>()?;
        let mut buffer: Box<[u8]> = vec![0; length as usize].into_boxed_slice();
        stream.read_exact(&mut buffer)?;
        Ok(Self {
            format,
            width,
            height,
            data: buffer.to_vec(),
        })
    }

    #[must_use]
    /// Get the image from the `MipMap`
    ///
    /// # Panics
    /// Panics if the `MipMap` is invalid
    pub fn get_image(&self) -> image::DynamicImage {
        let data = &*self.data;
        let mut width_2 = self.width;
        let compress = if width_2 % 32768 == width_2 {
            false
        } else {
            width_2 -= 32768;
            true
        };
        let mut img_size: u32 = u32::from(width_2) * u32::from(self.height);
        if self.format == Format::Bc1 {
            img_size /= 2;
        }
        let mut buffer: Box<[u8]> = vec![0; img_size as usize].into_boxed_slice();
        let mut out_buffer = vec![0u8; 4 * (width_2 as usize) * (self.height as usize)];
        if compress {
            hemtt_lzo::decompress_to_slice(data, &mut buffer);
            self.format.decompress(
                &buffer,
                usize::from(width_2),
                usize::from(self.height),
                &mut out_buffer,
            );
        } else {
            self.format.decompress(
                data,
                usize::from(width_2),
                usize::from(self.height),
                &mut out_buffer,
            );
        };
        image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(u32::from(width_2), u32::from(self.height), out_buffer)
                .unwrap(),
        )
    }
}
