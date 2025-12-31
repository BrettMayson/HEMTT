use std::{fmt::Display, io::Read};

use texpresso::{COLOUR_WEIGHTS_PERCEPTUAL, Format, Params};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PaXType {
    DXT1,
    DXT2,
    DXT3,
    DXT4,
    DXT5,
    ARGB4,
    ARGBA5,
    ARGB8,
    GRAYA,
}

impl PaXType {
    pub fn from_stream<I: Read>(stream: &mut I) -> Option<Self> {
        let mut bytes = [0; 2];
        if stream.read_exact(&mut bytes).is_ok() {
            Self::from_bytes(bytes)
        } else {
            None
        }
    }

    #[must_use]
    pub const fn from_bytes(bytes: [u8; 2]) -> Option<Self> {
        match bytes {
            [1, 255] => Some(Self::DXT1),    // 0x01FF
            [2, 255] => Some(Self::DXT2),    // 0x02FF
            [3, 255] => Some(Self::DXT3),    // 0x03FF
            [4, 255] => Some(Self::DXT4),    // 0x04FF
            [5, 255] => Some(Self::DXT5),    // 0x05FF
            [68, 68] => Some(Self::ARGB4),   // 0x4444
            [85, 21] => Some(Self::ARGBA5),  // 0x1555
            [136, 136] => Some(Self::ARGB8), // 0x8888
            [128, 128] => Some(Self::GRAYA), // 0x8080
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_bytes(&self) -> [u8; 2] {
        match self {
            Self::DXT1 => [1, 255],
            Self::DXT2 => [2, 255],
            Self::DXT3 => [3, 255],
            Self::DXT4 => [4, 255],
            Self::DXT5 => [5, 255],
            Self::ARGB4 => [68, 68],
            Self::ARGBA5 => [85, 21],
            Self::ARGB8 => [136, 136],
            Self::GRAYA => [128, 128],
        }
    }

    #[must_use]
    pub const fn as_u32(&self) -> u32 {
        match self {
            // Self::P8 => 0,
            Self::GRAYA => 1,
            // Self::RGB565 => 2,
            Self::ARGBA5 => 3,
            Self::ARGB4 => 4,
            Self::ARGB8 => 5,
            Self::DXT1 => 6,
            Self::DXT2 => 7,
            Self::DXT3 => 8,
            Self::DXT4 => 9,
            Self::DXT5 => 10,
        }
    }

    #[must_use]
    pub const fn from_u32(value: u32) -> Option<Self> {
        match value {
            1 => Some(Self::GRAYA),
            3 => Some(Self::ARGBA5),
            4 => Some(Self::ARGB4),
            5 => Some(Self::ARGB8),
            6 => Some(Self::DXT1),
            7 => Some(Self::DXT2),
            8 => Some(Self::DXT3),
            9 => Some(Self::DXT4),
            10 => Some(Self::DXT5),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_dxt(&self) -> bool {
        matches!(
            self,
            Self::DXT1 | Self::DXT2 | Self::DXT3 | Self::DXT4 | Self::DXT5
        )
    }

    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn image_size(&self, width: usize, height: usize) -> usize {
        match *self {
            Self::DXT1 | Self::DXT2 | Self::DXT3 | Self::DXT4 | Self::DXT5 => {
                let format: Format = (*self).try_into().expect("DXT formats are valid");
                format.compressed_size(width, height)
            }
            Self::ARGBA5 | Self::ARGB4 => width * height * 2,
            Self::ARGB8 => width * height * 4,
            Self::GRAYA => width * height,
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn compress(&self, data: &[u8], width: usize, height: usize, output: &mut [u8]) {
        match *self {
            Self::DXT1 | Self::DXT3 | Self::DXT5 => {
                let format: Format = (*self).try_into().expect("DXT formats are valid");
                format.compress(
                    data,
                    width,
                    height,
                    Params {
                        algorithm: texpresso::Algorithm::IterativeClusterFit,
                        weights: COLOUR_WEIGHTS_PERCEPTUAL,
                        weigh_colour_by_alpha: true,
                    },
                    output,
                );
            }
            Self::DXT2 | Self::DXT4 => {
                unimplemented!()
            }
            Self::ARGBA5 => {
                // convert from RGBA8 to ARGB1555
                let max_pixels = std::cmp::min(
                    width * height,
                    std::cmp::min(data.len() / 4, output.len() / 2),
                );
                for i in 0..max_pixels {
                    let offset = i * 2; // ARGB1555 uses 2 bytes per pixel
                    let r = u16::from(data[i * 4] >> 3); // R (5 bits)
                    let g = u16::from(data[i * 4 + 1] >> 3); // G (5 bits)
                    let b = u16::from(data[i * 4 + 2] >> 3); // B (5 bits)
                    let a = u16::from(data[i * 4 + 3] >= 128); // A (1 bit)
                    let pixel = (a << 15) | (r << 10) | (g << 5) | b;
                    output[offset] = (pixel & 0xFF) as u8;
                    output[offset + 1] = (pixel >> 8) as u8;
                }
            }
            Self::ARGB4 => {
                // convert from RGBA8 to ARGB4444
                let max_pixels = std::cmp::min(
                    width * height,
                    std::cmp::min(data.len() / 4, output.len() / 2),
                );
                for i in 0..max_pixels {
                    let offset = i * 2; // ARGB4444 uses 2 bytes per pixel
                    let r = u16::from(data[i * 4] >> 4); // R (4 bits)
                    let g = u16::from(data[i * 4 + 1] >> 4); // G (4 bits)
                    let b = u16::from(data[i * 4 + 2] >> 4); // B (4 bits)
                    let a = u16::from(data[i * 4 + 3] >> 4); // A (4 bits)
                    let pixel = (a << 12) | (b << 8) | (g << 4) | r;
                    output[offset] = (pixel & 0xFF) as u8;
                    output[offset + 1] = (pixel >> 8) as u8;
                }
            }
            Self::ARGB8 => {
                // convert from RGBA8 to ARGB8888
                let max_pixels = std::cmp::min(
                    width * height,
                    std::cmp::min(data.len() / 4, output.len() / 4),
                );
                for i in 0..max_pixels {
                    let offset = i * 4; // Each pixel is 4 bytes
                    output[offset] = data[i * 4 + 2]; // B
                    output[offset + 1] = data[i * 4 + 1]; // G
                    output[offset + 2] = data[i * 4]; // R
                    output[offset + 3] = data[i * 4 + 3]; // A
                }
            }
            Self::GRAYA => {
                // convert from RGBA8 to GRAY8
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                {
                    let max_pixels = std::cmp::min(
                        width * height,
                        std::cmp::min(data.len() / 4, output.len()),
                    );
                    for i in 0..max_pixels {
                        let r = u16::from(data[i * 4]);
                        let g = u16::from(data[i * 4 + 1]);
                        let b = u16::from(data[i * 4 + 2]);
                        // Use luminosity method to calculate grayscale value
                        let gray = 0.07f32.mul_add(
                            f32::from(b),
                            0.21f32.mul_add(f32::from(r), 0.72 * f32::from(g)),
                        ) as u8;
                        output[i] = gray;
                    }
                }
            }
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn decompress(&self, data: &[u8], width: usize, height: usize, output: &mut [u8]) {
        match *self {
            Self::DXT1 | Self::DXT3 | Self::DXT5 => {
                let format: Format = (*self).try_into().expect("DXT formats are valid");
                format.decompress(data, width, height, output);
            }
            Self::DXT2 | Self::DXT4 => {
                unimplemented!()
            }
            #[allow(clippy::cast_possible_truncation)]
            Self::ARGBA5 => {
                // convert from ARGB1555 to RGBA8
                for i in 0..(width * height) {
                    let offset = i * 2; // ARGB1555 uses 2 bytes per pixel
                    if offset + 1 < data.len() {
                        let pixel = u16::from_le_bytes([data[offset], data[offset + 1]]);
                        output[i * 4] = (((pixel >> 10) & 0x1F) << 3) as u8; // R (5 bits)
                        output[i * 4 + 1] = (((pixel >> 5) & 0x1F) << 3) as u8; // G (5 bits)
                        output[i * 4 + 2] = ((pixel & 0x1F) << 3) as u8; // B (5 bits)
                        output[i * 4 + 3] = if (pixel & 0x8000) != 0 { 255 } else { 0 }; // A (1 bit)
                    }
                }
            }
            #[allow(clippy::cast_possible_truncation)]
            Self::ARGB4 => {
                // convert from ARGB4444 to RGBA8
                for i in 0..(width * height) {
                    let offset = i * 2; // ARGB4444 uses 2 bytes per pixel
                    if offset + 1 < data.len() {
                        let pixel = u16::from_le_bytes([data[offset], data[offset + 1]]);
                        output[i * 4] = ((pixel & 0x0F) << 4) as u8; // R (4 bits)
                        output[i * 4 + 1] = (((pixel >> 4) & 0x0F) << 4) as u8; // G (4 bits)
                        output[i * 4 + 2] = (((pixel >> 8) & 0x0F) << 4) as u8; // B (4 bits)
                        output[i * 4 + 3] = (((pixel >> 12) & 0x0F) << 4) as u8; // A (4 bits)
                    }
                }
            }
            #[allow(clippy::cast_possible_truncation)]
            Self::ARGB8 => {
                // convert from ARGB8888 to RGBA8
                for i in 0..(width * height) {
                    let offset = i * 4; // Each pixel is 4 bytes
                    if offset + 3 < data.len() {
                        output[i * 4] = data[offset + 2]; // R
                        output[i * 4 + 1] = data[offset + 1]; // G
                        output[i * 4 + 2] = data[offset]; // B
                        output[i * 4 + 3] = data[offset + 3]; // A
                    }
                }
            }
            Self::GRAYA => {
                // convert from GRAY8 to RGBA8
                for i in 0..(width * height) {
                    if i < data.len() {
                        let pixel = data[i];
                        output[i * 4] = pixel; // R
                        output[i * 4 + 1] = pixel; // G
                        output[i * 4 + 2] = pixel; // B
                        output[i * 4 + 3] = 0xFF; // A (full opacity)
                    }
                }
            }
        }
    }
}

impl TryFrom<PaXType> for Format {
    type Error = ();
    fn try_from(pax: PaXType) -> Result<Self, Self::Error> {
        match pax {
            PaXType::DXT1 => Ok(Self::Bc1),
            PaXType::DXT3 => Ok(Self::Bc2),
            PaXType::DXT5 => Ok(Self::Bc3),
            _ => Err(()),
        }
    }
}

impl From<Format> for PaXType {
    fn from(pax: Format) -> Self {
        match pax {
            Format::Bc1 => Self::DXT1,
            Format::Bc2 => Self::DXT3,
            Format::Bc3 => Self::DXT5,
            _ => unimplemented!(),
        }
    }
}

impl Display for PaXType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DXT1 => write!(f, "DXT1"),
            Self::DXT2 => write!(f, "DXT2"),
            Self::DXT3 => write!(f, "DXT3"),
            Self::DXT4 => write!(f, "DXT4"),
            Self::DXT5 => write!(f, "DXT5"),
            Self::ARGB4 => write!(f, "ARGB4444"),
            Self::ARGBA5 => write!(f, "ARGB1555"),
            Self::ARGB8 => write!(f, "ARGB8888"),
            Self::GRAYA => write!(f, "GRAYA"),
        }
    }
}
