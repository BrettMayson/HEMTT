#![allow(clippy::cast_possible_truncation)]

use std::{
    io::{Error, Read, Seek, SeekFrom, Write},
    mem::size_of,
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use indexmap::IndexMap;

use crate::{MipMap, PaXType};

#[derive(Debug)]
pub struct Paa {
    format: PaXType,
    taggs: IndexMap<String, Vec<u8>>,
    maps: Vec<(MipMap, u64)>,
}

impl Paa {
    #[must_use]
    pub fn new(format: PaXType) -> Self {
        Self {
            format,
            taggs: IndexMap::new(),
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
    pub const fn taggs(&self) -> &IndexMap<String, Vec<u8>> {
        &self.taggs
    }

    #[must_use]
    /// Get the maps of the Paa
    pub const fn maps(&self) -> &Vec<(MipMap, u64)> {
        &self.maps
    }

    /// Read the Paa from the given input
    ///
    /// # Errors
    /// [`std::io::Error`] if the input is not readable, or the Paa is invalid
    pub fn read<I: Seek + Read>(mut input: I) -> Result<Self, Error> {
        let Some(pax_type) = PaXType::from_stream(&mut input) else {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid paa format",
            ));
        };
        let mut paa = Self::new(pax_type);
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
                        std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid tagg name")
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
                    paa.maps.push((
                        MipMap::from_stream(paa.format, &mut input)?,
                        u64::from(seek),
                    ));
                }
            }
        }
        Ok(paa)
    }

    /// Write the Paa to the given output
    ///
    /// # Errors
    /// [`std::io::Error`] if the output is not writable
    pub fn write<O: Seek + Write>(&self, output: &mut O) -> Result<(), Error> {
        output.write_all(&self.format().as_bytes())?;
        let mut offset = 2;
        for (name, data) in &self.taggs {
            if name == "SFFO" {
                continue;
            }
            output.write_all(b"GGAT")?;
            output.write_all(name.as_bytes())?;
            output.write_u32::<LittleEndian>(data.len() as u32)?;
            output.write_all(data)?;
            offset += 12 + data.len();
        }
        offset += 2; // For the index palette
        {
            output.write_all(b"GGAT")?;
            output.write_all(b"SFFO")?;
            output.write_u32::<LittleEndian>((16 * size_of::<u32>()) as u32)?;
            offset += 12 + (16 * size_of::<u32>());
            for (mipmap, _) in &self.maps {
                output.write_u32::<LittleEndian>(offset as u32)?;
                // u16 + u16 + u24 + data
                offset += mipmap.data().len() + 2 + 2 + 3;
            }
            for _ in self.maps.len()..16 {
                output.write_u32::<LittleEndian>(0)?;
            }
        }
        output.write_u16::<LittleEndian>(0)?; // Index palette
        for (mipmap, _) in &self.maps {
            mipmap.write(output)?;
        }
        output.write_u32::<LittleEndian>(0)?;
        output.write_u16::<LittleEndian>(0)?;
        Ok(())
    }

    /// Create a Paa from a `DynamicImage`
    ///
    /// # Errors
    /// [`std::io::Error`] if the image cannot be converted to the specified format
    #[cfg(feature = "generate")]
    pub fn from_dynamic(
        image: &image::DynamicImage,
        format: PaXType,
    ) -> Result<Self, std::io::Error> {
        let rgba_image = image.to_rgba8();
        let mipmap = MipMap::from_rgba_image(&rgba_image, format)?;
        let mut paa = Self::new(format);
        paa.maps.push((mipmap, 0));
        // Generate tags
        // - Average
        let mut has_transparency = false;
        {
            let avg_color = rgba_image
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
            let avg_color = [
                (avg_color[0] / (rgba_image.width() * rgba_image.height())) as u8,
                (avg_color[1] / (rgba_image.width() * rgba_image.height())) as u8,
                (avg_color[2] / (rgba_image.width() * rgba_image.height())) as u8,
                (avg_color[3] / (rgba_image.width() * rgba_image.height())) as u8,
            ];
            if avg_color[3] < 255 {
                has_transparency = true;
            }
            paa.taggs.insert(
                "CGVA".to_string(),
                vec![avg_color[0], avg_color[1], avg_color[2], avg_color[3]],
            );
        }
        // - Max
        {
            let max_color: [u8; 4] = [255, 255, 255, 255]; // always this value for some reason
            paa.taggs.insert(
                "CXAM".to_string(),
                vec![max_color[0], max_color[1], max_color[2], max_color[3]],
            );
        }
        if has_transparency {
            // - Alpha flag
            paa.taggs.insert("GALF".to_string(), vec![1, 0, 0, 0]);
        }
        // Generate mipmaps for DXT formats
        if format.is_dxt() {
            let mut width = rgba_image.width();
            let mut height = rgba_image.height();
            while width > 4 && height > 4 {
                width = (width / 2).max(1);
                height = (height / 2).max(1);
                let mipmap = MipMap::from_rgba_image(
                    &image::imageops::resize(
                        &rgba_image,
                        width,
                        height,
                        image::imageops::FilterType::Lanczos3,
                    ),
                    format,
                )?;
                paa.maps.push((mipmap, 0));
            }
        }
        Ok(paa)
    }

    #[cfg(feature = "json")]
    /// Get the Paa as a JSON string
    ///
    /// # Errors
    /// [`String`] if the Paa is invalid
    pub fn json(&self) -> Result<String, String> {
        serde_json::to_string(&PaaJson {
            format: self.format.to_string(),
            maps: self.maps.iter().map(|(mipmap, _)| mipmap.json()).collect(),
        })
        .map_err(|e| e.to_string())
    }
}

#[cfg(feature = "json")]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PaaJson {
    format: String,
    maps: Vec<String>,
}
