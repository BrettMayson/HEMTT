use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug)]
pub struct MipMap {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
    format: image::dxt::DXTVariant,
}

impl MipMap {
    pub fn from_stream<I: Read>(
        format: image::dxt::DXTVariant,
        stream: &mut I,
    ) -> Result<Self, std::io::Error> {
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

    pub fn get_image(&self) -> image::DynamicImage {
        let data = &*self.data;
        let mut width_2 = self.width;
        let compress = if width_2 % 32768 == width_2 {
            false
        } else {
            width_2 -= 32768;
            true
        };
        let mut img_size: u32 = (width_2 as u32) * (self.height as u32);
        if self.format == image::dxt::DXTVariant::DXT1 {
            img_size /= 2;
        }
        let mut buffer: Box<[u8]> = vec![0; img_size as usize].into_boxed_slice();
        let decoder = if compress {
            crate::lzo::LzoContext::decompress_to_slice(data, &mut buffer).unwrap();
            image::dxt::DxtDecoder::new(&*buffer, width_2 as u32, self.height as u32, self.format)
                .unwrap()
        } else {
            image::dxt::DxtDecoder::new(data, width_2 as u32, self.height as u32, self.format)
                .unwrap()
        };
        image::DynamicImage::from_decoder(decoder).unwrap()
    }
}
