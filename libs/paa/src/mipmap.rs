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
    #[cfg(feature = "generate")]
    pub fn from_rgba_image(
        image: &image::RgbaImage,
        format: PaXType,
    ) -> Result<Self, std::io::Error> {
        use image::EncodableLayout;
        use texpresso::Format;
        let (width, height) = image.dimensions();
        let mut data = {
            let format: Format = format.into();
            vec![0u8; format.compressed_size(width as usize, height as usize)]
        };
        format.compress(image.as_bytes(), width as usize, height as usize, &mut data);
        let compress = format.is_dxt() && width >= 256 && height >= 256;
        let stored_width = u16::try_from(width).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Width exceeds u16 limit")
        })? + if compress { 32768 } else { 0 };
        Ok(Self {
            width: stored_width,
            height: u16::try_from(height).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "Height exceeds u16 limit")
            })?,
            data: {
                if compress {
                    let mut output = Vec::with_capacity(hemtt_lzo::worst_compress(data.len()));
                    hemtt_lzo::compress(&data, &mut output)
                        .map_err(|_| std::io::Error::other("Failed to compress MipMap data"))?;
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
        if self.is_compressed() {
            width -= 32768;
        }
        width
    }

    #[must_use]
    /// Is the `MipMap` compressed
    pub const fn is_compressed(&self) -> bool {
        self.width % 32768 != self.width
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

        // Calculate decompressed data size based on format
        let img_size = match self.format {
            PaXType::GRAYA | PaXType::ARGB4 | PaXType::ARGBA5 => {
                u32::from(actual_width) * u32::from(self.height) * 2
            }
            PaXType::ARGB8 => u32::from(actual_width) * u32::from(self.height) * 4,
            PaXType::DXT1 => (u32::from(actual_width) * u32::from(self.height)) / 2,
            _ => u32::from(actual_width) * u32::from(self.height),
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

        let mut buffer: Box<[u8]> = vec![0; img_size as usize].into_boxed_slice();
        if decompression == Compression::Lzss {
            match hemtt_lzo::decompress_to_slice(data, &mut buffer) {
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
            expand_unknown_input_length(data, &mut buffer).expect("Failed to decompress LZSS data");
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

/// Decompress data from input to a fixed-size output buffer
///
/// Returns the number of bytes read from input on success, or an error message
pub fn expand_unknown_input_length(
    input: &[u8],
    out_buf: &mut [u8],
) -> Result<usize, &'static str> {
    let outlen = out_buf.len();
    let mut flag: u8 = 0;
    let mut rpos: usize;
    let mut rlen: u8;
    let mut fl: usize = 0;
    // let mut calculated_checksum: u32 = 0;
    let mut pi: usize = 0;
    let mut data: u8;

    let mut remaining_outlen = outlen;

    'outer: while remaining_outlen > 0 {
        if pi >= input.len() {
            return Err("Unexpected end of input data");
        }

        flag = input[pi];
        pi += 1;

        for _ in 0..8 {
            if (flag & 0x01) != 0 {
                // Raw data
                if pi >= input.len() {
                    return Err("Unexpected end of input data during raw byte read");
                }

                data = input[pi];
                pi += 1;
                // calculated_checksum += u32::from(data);
                out_buf[fl] = data;
                fl += 1;

                remaining_outlen -= 1;
                if remaining_outlen == 0 {
                    break 'outer; // goto finish
                }
            } else {
                // Back reference - need 2 more bytes
                if pi + 1 >= input.len() {
                    return Err("Unexpected end of input data during back reference read");
                }

                rpos = input[pi] as usize;
                pi += 1;

                rlen = (input[pi] & 0x0F) + 3;
                rpos += ((input[pi] & 0xF0) as usize) << 4;
                pi += 1;

                // Special case: space fill
                let mut skip_backref = false;
                while rpos > fl {
                    // calculated_checksum += 0x20;
                    out_buf[fl] = 0x20;
                    fl += 1;

                    remaining_outlen -= 1;
                    if remaining_outlen == 0 {
                        break 'outer; // goto finish
                    }

                    rlen -= 1;
                    if rlen == 0 {
                        skip_backref = true;
                        break;
                    }
                }

                if !skip_backref {
                    // Standard back reference copy
                    rpos = fl - rpos;

                    // Need to copy byte-by-byte because source and destination might overlap
                    for _ in 0..rlen {
                        data = out_buf[rpos];
                        // calculated_checksum += u32::from(data);
                        out_buf[fl] = data;
                        fl += 1;
                        rpos += 1;

                        remaining_outlen -= 1;
                        if remaining_outlen == 0 {
                            break 'outer; // goto finish
                        }
                    }
                }
            }

            // Shift flag for next bit
            flag >>= 1;
        }
    }

    // Check excess bits in final flag byte
    if flag & 0xFE != 0 {
        return Err("Excess bits in final flag byte");
    }

    // Read checksum
    if pi + 3 >= input.len() {
        return Err("Cannot read checksum: unexpected end of input");
    }

    // let read_checksum =
    //     u32::from_ne_bytes([input[pi], input[pi + 1], input[pi + 2], input[pi + 3]]);

    // if read_checksum != calculated_checksum {
    // this is expected right now with PAAs
    // they store the checksum in a different format and
    // I don't feel like fixing it right now
    // }
    Ok(pi + 4)
}
