use std::io::{Read, Seek};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::PaXType;

#[derive(Debug)]
pub struct MipMap {
    width: u16,
    height: u16,
    data: Vec<u8>,
    format: PaXType,
}

impl MipMap {
    /// Read the `MipMap` from the given input
    ///
    /// # Errors
    /// [`std::io::Error`] if the input is not readable, or the `MipMap` is invalid
    pub fn from_stream<I: Seek + Read>(
        format: PaXType,
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

    /// Write the `MipMap` to the given output
    ///
    /// # Errors
    /// [`std::io::Error`] if the output is not writable
    pub fn write(&self, output: &mut impl std::io::Write) -> Result<(), std::io::Error> {
        output.write_u16::<LittleEndian>(self.width)?;
        output.write_u16::<LittleEndian>(self.height)?;
        output.write_u24::<LittleEndian>(u32::try_from(self.data.len()).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "data length exceeds 24-bit limit",
            )
        })?)?;
        output.write_all(&self.data)?;
        Ok(())
    }

    /// Create a `MipMap` from an RGBA image
    ///
    /// # Errors
    /// [`std::io::Error`] if the image cannot be converted to the specified format
    /// [`std::io::Error`] if the width or height exceed u16 limits
    /// [`std::io::Error`] if the width or height are not powers of two
    ///
    /// # Panics
    /// If the `PaXType` is not a valid DXT format
    #[cfg(feature = "generate")]
    pub fn from_rgba_image(
        image: &image::RgbaImage,
        format: PaXType,
    ) -> Result<Self, std::io::Error> {
        use image::EncodableLayout;
        let (width, height) = image.dimensions();
        let mut data = vec![0u8; format.image_size(width as usize, height as usize)];
        format.compress(image.as_bytes(), width as usize, height as usize, &mut data);
        let dxt_compress = format.is_dxt() && width >= 256 && height >= 256;
        let stored_width = u16::try_from(width).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Width exceeds u16 limit")
        })? + if dxt_compress { 32768 } else { 0 };
        Ok(Self {
            width: stored_width,
            height: u16::try_from(height).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "Height exceeds u16 limit")
            })?,
            data: {
                if dxt_compress {
                    let mut output =
                        Vec::with_capacity(hemtt_lzo::lzss::worst_compress(data.len()));
                    hemtt_lzo::lzss::compress(&data, &mut output)
                        .map_err(|_| std::io::Error::other("Failed to compress MipMap data"))?;
                    output
                } else if !format.is_dxt() {
                    let mut output = vec![0u8; data.len() * 2]; // some small data may expand when compressed
                    let size = hemtt_lzo::lz77::compress(&data, &mut output).map_err(|e| {
                        std::io::Error::other(format!("Failed to compress MipMap data: {e}"))
                    })?;
                    output.truncate(size);
                    output
                } else {
                    data
                }
            },
            format,
        })
    }

    #[must_use]
    /// Get the width of the `MipMap`
    pub const fn width(&self) -> u16 {
        let mut width = self.width;
        if self.is_compressed() && self.format.is_dxt() {
            width -= 32768;
        }
        width
    }

    #[must_use]
    /// Is the `MipMap` compressed
    pub const fn is_compressed(&self) -> bool {
        !self.format.is_dxt() || self.width % 32768 != self.width
    }

    #[must_use]
    /// Get the height of the `MipMap`
    pub const fn height(&self) -> u16 {
        self.height
    }

    #[must_use]
    /// Get the data of the `MipMap`
    pub const fn data(&self) -> &Vec<u8> {
        &self.data
    }

    #[must_use]
    /// Get the format of the `MipMap`
    pub const fn format(&self) -> &PaXType {
        &self.format
    }

    #[must_use]
    /// Get the format of the `MipMap` as a string
    pub fn format_display(&self) -> String {
        format!("{:?}", self.format)
    }

    #[must_use]
    /// Get the image from the `MipMap`
    ///
    /// # Panics
    /// Panics if the `MipMap` is invalid
    pub fn get_image(&self) -> image::DynamicImage {
        #[derive(Debug, PartialEq, Eq)]
        pub enum Compression {
            None,
            Lzss,
            Lz77,
        }
        let data = &*self.data;

        // Get actual width - for DXT formats, check compression flag in width
        let actual_width = if self.format.is_dxt() && self.width % 32768 != self.width {
            self.width - 32768
        } else {
            self.width
        };

        // Output buffer is always RGBA8 (4 bytes per pixel)
        let mut out_buffer = vec![0u8; 4 * (actual_width as usize) * (self.height as usize)];

        // Determine if we need to decompress
        let decompression = if !self.format.is_dxt() {
            Compression::Lz77
        } else if self.width % 32768 != self.width {
            Compression::Lzss
        } else {
            Compression::None
        };

        let mut buffer: Box<[u8]> =
            vec![
                0;
                self.format
                    .image_size(actual_width as usize, self.height as usize)
            ]
            .into_boxed_slice();
        if decompression == Compression::Lzss {
            match hemtt_lzo::lzss::decompress_to_slice(data, &mut buffer) {
                Ok(decompressed) => {
                    self.format.decompress(
                        decompressed,
                        usize::from(actual_width),
                        usize::from(self.height),
                        &mut out_buffer,
                    );
                }
                Err(e) => {
                    eprintln!(
                        "Failed to decompress LZSS data for {:?} ({}x{}): {}",
                        self.format, actual_width, self.height, e
                    );
                    self.format.decompress(
                        data,
                        usize::from(actual_width),
                        usize::from(self.height),
                        &mut out_buffer,
                    );
                }
            }
        } else if decompression == Compression::Lz77 {
            hemtt_lzo::lz77::decompress(data, &mut buffer).expect("Failed to decompress LZ77 data");
            self.format.decompress(
                &buffer,
                usize::from(actual_width),
                usize::from(self.height),
                &mut out_buffer,
            );
        } else {
            self.format.decompress(
                data,
                usize::from(actual_width),
                usize::from(self.height),
                &mut out_buffer,
            );
        }
        image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(u32::from(actual_width), u32::from(self.height), out_buffer)
                .expect("paa should contain valid image data"),
        )
    }

    #[cfg(feature = "json")]
    #[must_use]
    /// Returns the image as a base64 encoded string
    ///
    /// # Panics
    /// Panics if the image cannot be encoded
    pub fn json(&self) -> String {
        use base64::Engine as _;
        let img = self.get_image();
        let mut buffer = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buffer, image::ImageFormat::Png)
            .expect("Failed to write PNG");
        base64::prelude::BASE64_STANDARD.encode(buffer.get_ref())
    }
}
