#![allow(clippy::cast_possible_truncation)]

use std::{
    io::{Error, Read, Seek, SeekFrom, Write},
    mem::size_of,
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use image::{EncodableLayout, RgbaImage};
use texpresso::{Format, Params};

use crate::{MipMap, PaXType};

use std::collections::HashMap;

#[derive(Debug)]
pub struct Paa {
    format: PaXType,
    taggs: HashMap<String, Vec<u8>>,
    maps: Vec<MipMap>,
}

impl Paa {
    #[must_use]
    pub fn new(format: PaXType) -> Self {
        Self {
            format,
            taggs: HashMap::new(),
            maps: Vec::new(),
        }
    }

    #[must_use]
    /// Get the format of the Paa
    pub const fn format(&self) -> &PaXType {
        &self.format
    }

    #[must_use]
    /// Get the taggs of the Paa
    pub const fn taggs(&self) -> &HashMap<String, Vec<u8>> {
        &self.taggs
    }

    #[must_use]
    /// Get the maps of the Paa
    pub const fn maps(&self) -> &Vec<MipMap> {
        &self.maps
    }

    /// Read the Paa from the given input
    ///
    /// # Errors
    /// [`std::io::Error`] if the input is not readable, or the Paa is invalid
    pub fn read<I: Seek + Read>(mut input: I) -> Result<Self, Error> {
        if let Some(pax) = PaXType::from_stream(&mut input) {
            let mut paa = Self::new(pax);
            // Read Taggs
            while {
                let mut tagg_sig = [0; 4];
                input.read_exact(&mut tagg_sig)?;
                std::str::from_utf8(&tagg_sig) == Ok("GGAT")
            } {
                let name = {
                    let mut bytes = [0; 4];
                    input.read_exact(&mut bytes)?;
                    std::str::from_utf8(&bytes)
                        .map_err(|_| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "invalid tagg name",
                            )
                        })?
                        .to_string()
                };
                paa.taggs.insert(name, {
                    let mut buffer: Box<[u8]> =
                        vec![0; (input.read_u32::<LittleEndian>()?) as usize].into_boxed_slice();
                    input.read_exact(&mut buffer)?;
                    buffer.to_vec()
                });
            }
            // Read MipMaps
            if let Some(offs) = paa.taggs.get("SFFO") {
                for i in 0..(offs.len() / 4) {
                    let mut seek: [u8; 4] = [0; 4];
                    let p = i * 4;
                    seek.clone_from_slice(&offs[p..p + 4]);
                    let seek = u32::from_le_bytes(seek);
                    if seek != 0 {
                        input.seek(SeekFrom::Start(u64::from(seek)))?;
                        paa.maps
                            .push(MipMap::from_stream(paa.format.clone().into(), &mut input)?);
                    }
                }
            }
            Ok(paa)
        } else {
            Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid paa format",
            ))
        }
    }

    /// Write the Paa to the given output
    ///
    /// # Errors
    /// [`std::io::Error`] if the output is not writable
    pub fn write(image: &RgbaImage, output: &mut impl Write) -> Result<(), Error> {
        let algo: Format = PaXType::DXT5.clone().into();
        output.write_all(&PaXType::DXT5.as_bytes())?; // 2

        // Average Color
        output.write_all(b"GGATAVGC")?; // 8
        output.write_u32::<LittleEndian>(size_of::<u32>() as u32)?; // 4
        let avg_color = image
            .pixels()
            .map(|p| {
                [
                    u32::from(p.0[0]),
                    u32::from(p.0[1]),
                    u32::from(p.0[2]),
                    u32::from(p.0[3]),
                ]
            })
            .fold([0, 0, 0, 0], |mut acc, p| {
                acc[0] += p[0];
                acc[1] += p[1];
                acc[2] += p[2];
                acc[3] += p[3];
                acc
            });
        output.write_u32::<LittleEndian>(u32::from_le_bytes([
            (avg_color[0] / (image.width() * image.height())) as u8,
            (avg_color[1] / (image.width() * image.height())) as u8,
            (avg_color[2] / (image.width() * image.height())) as u8,
            (avg_color[3] / (image.width() * image.height())) as u8,
        ]))?; // 4

        // Max Color
        output.write_all(b"GGATCXAM")?; // 8
        output.write_u32::<LittleEndian>(size_of::<u32>() as u32)?; // 4
        let max_color = image
            .pixels()
            .map(|p| p.0)
            .fold([0, 0, 0, 0], |mut acc, p| {
                acc[0] = acc[0].max(p[0]);
                acc[1] = acc[1].max(p[1]);
                acc[2] = acc[2].max(p[2]);
                acc[3] = acc[3].max(p[3]);
                acc
            });
        output.write_u32::<LittleEndian>(u32::from_le_bytes(max_color))?; // 4

        // Offset Table
        output.write_all(b"GGATSFFO")?; // 8
        output.write_u32::<LittleEndian>(16 * size_of::<u32>() as u32)?; // 4
        let pos = 2 + 8 + 4 + 4 + 8 + 4 + 4 + 8 + 4 + (16 * 4);
        output.write_u32::<LittleEndian>(pos)?; // 4
        for _ in 0..15 {
            // 15 * 4
            output.write_u32::<LittleEndian>(0)?;
        }

        // Write main image
        output.write_u16::<LittleEndian>(image.width() as u16)?;
        output.write_u16::<LittleEndian>(image.height() as u16)?;
        let size = algo.compressed_size(image.width() as usize, image.height() as usize);
        output.write_u24::<LittleEndian>(size as u32)?;
        let mut buffer = vec![0; size];
        algo.compress(
            image.as_bytes(),
            image.width() as usize,
            image.height() as usize,
            Params::default(),
            &mut buffer,
        );
        output.write_all(&buffer)?;

        Ok(())
    }
}
