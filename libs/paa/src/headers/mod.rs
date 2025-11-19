use std::{
    io::{Error, Read, Seek, Write},
    path::{Path, PathBuf},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use hemtt_common::io::{ReadExt, WriteExt};

use crate::{PaXType, Paa};

// struct TexHeader.bin
// {
//  MimeType "0DHT" ;    // NOT asciiz. '0' = 0x30. This is mimetype "TexHeaDer0"
//  ulong    version;    // 1
//  ulong    nTextures;  //
//  TexBody  TexBodies[nTextures];
// };
#[derive(Debug, PartialEq, Eq)]
pub struct Headers {
    textures: Vec<TextureHeader>,
}

impl Headers {
    #[must_use]
    /// Create a new Headers instance
    pub fn new(textures: Vec<TextureHeader>) -> Self {
        Self { textures }
    }

    /// Read the Header from the given input
    ///
    /// # Errors
    /// [`std::io::Error`] if the input is not readable, or the Header is invalid
    ///
    /// # Panics
    /// - Panics if the `TexHeader` version is not 1
    /// - Panics if the mime type is invalid
    pub fn read<I: Seek + Read>(mut input: I) -> Result<Self, Error> {
        let mut mime_type = [0; 4];
        input.read_exact(&mut mime_type)?;
        if &mime_type != b"0DHT" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid TexHeader mime type",
            ));
        }

        let version = input.read_u32::<LittleEndian>()?;
        assert!(
            version == 1,
            "unsupported TexHeader version: {version}, expected 1"
        );
        let n_textures = input.read_u32::<LittleEndian>()?;

        let mut textures = Vec::new();
        for _ in 0..n_textures {
            textures.push(TextureHeader::read(&mut input)?);
        }
        Ok(Self { textures })
    }

    /// Write the Header to the given output
    ///
    /// # Errors
    /// [`std::io::Error`] if the output is not writable
    ///
    /// # Panics
    /// - Panics if there are too many textures to write
    pub fn write(&self, output: &mut impl Write) -> Result<(), Error> {
        output.write_all(b"0DHT")?;
        output.write_u32::<LittleEndian>(1)?;
        output.write_u32::<LittleEndian>(
            u32::try_from(self.textures.len()).expect("too many textures"),
        )?;
        for texture in &self.textures {
            texture.write(output)?;
        }
        Ok(())
    }
}

// TexBody
// {
//  ulong          nColorPallets;      //always 1
//  ulong          Pallet_ptr;         //Always0 (there are none)
//  floats         AverageColor[r,g,b,a];//AVGCTAGG floating-point equivalent.
//  bytes          AverageColor[b,g,r,a];//AVGCTAGG in PAx file
//  bytes          MaxColor[b,g,r,a];    //MAXCTAGG in PAx file
//  ulong          clampflags;         // always 0
//  ulong          transparentColor;   // always 0xFFFFFFFF
//  byteBool       has_maxCtagg;       // the MaxColor was set by the paa
//                 isAlpha;	     // set if FLAGTAG=1: 'basic transparency'
//                 isTransparent;	     // set if FLAGTAG=2: 'alpha channel is not interpolated'
//                 isAlphaNonOpaque;   // set if isalpha, AND AverageColor alpha <0x80
//  ulong          nMipmaps;           // always same as nMipmapsCopy below
//  ulong          pax_format;         // see below Dxt1,2,3 etc
//  byteBool       littleEndian;       // Always true;
//  byte           isPaa; 	     //file was a .paa not .pac
//  Asciiz         PaaFile[];          // Relative to the file location of the texheader itself.
//                                     //"data\icons\m4a3_cco_ca.paa"
//                                     //"fnfal\data\fnfal_smdi.paa"
//  ulong          pax_suffix_type;    // _co, _ca, smdi, etc
//  ulong          nMipmapsCopy;       // same as nMipmaps above
//  MipMap         MipMaps[nMipmaps];  // see below
//  ulong          SizeOfPaxFile;      //
// };
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, PartialEq, Eq)]
pub struct TextureHeader {
    pub average_color: [u8; 4],
    pub max_color: [u8; 4],
    pub has_max_ctagg: bool,
    pub is_alpha: bool,
    pub is_transparent: bool,
    pub is_alpha_non_opaque: bool,
    pub pax_format: PaXType,
    pub is_paa: bool,
    pub paa_file: String,
    pub pax_suffix_type: u32,
    pub mipmaps: Vec<MipMap>,
    pub size_of_pax_file: u32,
}

impl TextureHeader {
    /// Read the `TextureHeader` from the given input
    ///
    /// # Errors
    /// [`std::io::Error`] if the input is not readable, or the `TextureHeader` is invalid
    pub fn read<I: Seek + Read>(mut input: I) -> Result<Self, Error> {
        let _n_color_pallets = input.read_u32::<LittleEndian>()?;
        let _pallet_ptr = input.read_u32::<LittleEndian>()?;
        let _average_color_floats = [
            input.read_f32::<LittleEndian>()?,
            input.read_f32::<LittleEndian>()?,
            input.read_f32::<LittleEndian>()?,
            input.read_f32::<LittleEndian>()?,
        ];
        let mut average_color = [0; 4];
        input.read_exact(&mut average_color)?;
        let mut max_color = [0; 4];
        input.read_exact(&mut max_color)?;
        let _clamp_flags = input.read_u32::<LittleEndian>()?;
        let _transparent_color = input.read_u32::<LittleEndian>()?;
        let has_max_ctagg = input.read_u8()? != 0;
        let is_alpha = input.read_u8()? != 0;
        let is_transparent = input.read_u8()? != 0;
        let is_alpha_non_opaque = input.read_u8()? != 0;
        let n_mipmaps = input.read_u32::<LittleEndian>()?;
        let pax_format = PaXType::from_u32(input.read_u32::<LittleEndian>()?).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid pax_format in TextureHeader",
            )
        })?;
        let _little_endian = input.read_u8()? != 0;
        let is_paa = input.read_u8()? != 0;
        let paa_file = input.read_cstring()?;
        let pax_suffix_type = input.read_u32::<LittleEndian>()?;
        let n_mipmaps_copy = input.read_u32::<LittleEndian>()?;
        if n_mipmaps != n_mipmaps_copy {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "nMipmaps and nMipmapsCopy do not match",
            ));
        }
        let mut mipmaps = Vec::new();
        for _ in 0..n_mipmaps {
            mipmaps.push(MipMap::read(&mut input)?);
        }
        let size_of_pax_file = input.read_u32::<LittleEndian>()?; // SizeOfPaxFile

        Ok(Self {
            average_color,
            max_color,
            has_max_ctagg,
            is_alpha,
            is_transparent,
            is_alpha_non_opaque,
            pax_format,
            is_paa,
            paa_file,
            pax_suffix_type,
            mipmaps,
            size_of_pax_file,
        })
    }

    /// Write the `TextureHeader` to the given output
    ///
    /// # Errors
    /// [`std::io::Error`] if the output is not writable
    ///
    /// # Panics
    /// - Panics if there are too many textures to write
    pub fn write(&self, output: &mut impl Write) -> Result<(), Error> {
        output.write_u32::<LittleEndian>(1)?; // nColorPallets
        output.write_u32::<LittleEndian>(0)?; // Pallet_ptr
        // AverageColor floats in [r, g, b, a] order (bytes are stored as [b, g, r, a])
        output.write_f32::<LittleEndian>(f32::from(self.average_color[2]) / 255.0)?; // r
        output.write_f32::<LittleEndian>(f32::from(self.average_color[1]) / 255.0)?; // g
        output.write_f32::<LittleEndian>(f32::from(self.average_color[0]) / 255.0)?; // b
        output.write_f32::<LittleEndian>(f32::from(self.average_color[3]) / 255.0)?; // a
        for &color in &self.average_color {
            output.write_u8(color)?; // AverageColor bytes
        }
        for &color in &self.max_color {
            output.write_u8(color)?; // MaxColor bytes
        }
        output.write_u32::<LittleEndian>(0)?; // clampflags
        output.write_u32::<LittleEndian>(0xFFFF_FFFF)?; // transparentColor
        output.write_u8(u8::from(self.has_max_ctagg))?; // has_maxCtagg
        output.write_u8(u8::from(self.is_alpha))?; // isAlpha
        output.write_u8(u8::from(self.is_transparent))?; // isTransparent
        output.write_u8(u8::from(self.is_alpha_non_opaque))?; // isAlphaNonOpaque
        output.write_u32::<LittleEndian>(
            u32::try_from(self.mipmaps.len()).expect("too many mipmaps"),
        )?; // nMipmaps
        output.write_u32::<LittleEndian>(self.pax_format.as_u32())?; // pax_format
        output.write_u8(1)?; // littleEndian
        output.write_u8(u8::from(self.is_paa))?; // isPaa
        output.write_cstring(&self.paa_file)?; // PaaFile
        output.write_u32::<LittleEndian>(self.pax_suffix_type)?; // pax_suffix_type
        output.write_u32::<LittleEndian>(
            u32::try_from(self.mipmaps.len()).expect("too many mipmaps"),
        )?; // nMipmapsCopy
        for mipmap in &self.mipmaps {
            MipMap::write(mipmap, output)?;
        }
        output.write_u32::<LittleEndian>(self.size_of_pax_file)?; // SizeOfPaxFile
        Ok(())
    }

    /// Create a `TextureHeader` from a PAA file
    ///
    /// # Errors
    /// [`std::io::Error`] if the PAA file cannot be opened or read
    ///
    /// # Panics
    /// - Panics if the PAA file has an invalid path
    pub fn from_file(root: &Path, path: &PathBuf) -> Result<Self, Error> {
        let paa = Paa::read(std::fs::File::open(path)?)?;
        let flag = paa.taggs().get("GALF").map_or(0, |flag| {
            let mut bytes: [u8; 4] = [0; 4];
            bytes.copy_from_slice(&flag[0..4]);
            u32::from_le_bytes(bytes)
        });
        let average_color = paa.taggs().get("CGVA").map_or([0; 4], |avg| {
            let mut bytes: [u8; 4] = [0; 4];
            bytes.copy_from_slice(&avg[0..4]);
            bytes
        });
        Ok(Self {
            average_color,
            max_color: paa.taggs().get("CXAM").map_or([0; 4], |max| {
                let mut bytes: [u8; 4] = [0; 4];
                bytes.copy_from_slice(&max[0..4]);
                bytes
            }),
            has_max_ctagg: paa.taggs().contains_key("CXAM"),
            is_alpha: flag == 1,
            is_transparent: flag == 2,
            is_alpha_non_opaque: flag == 1 && average_color[3] < 0x80,
            is_paa: {
                let ext = path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_lowercase();
                ext == "paa"
            },
            pax_format: paa.format().to_owned(),
            paa_file: {
                let relative_path = path
                    .strip_prefix(root)
                    .expect("Failed to get relative path");
                relative_path.to_string_lossy().to_string()
            },
            pax_suffix_type: {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map_or(0, |suffix| {
                        suffix.rfind('_').map_or(0, |pos| {
                            if path
                                .file_stem()
                                .expect("stem must exist")
                                .to_str()
                                .expect("stem must be valid UTF-8")
                                .ends_with("_ti_ca")
                            {
                                return 12;
                            }
                            let suffix_str = &suffix[pos + 1..];
                            if suffix_str.starts_with('n') {
                                return 3;
                            }
                            match suffix_str {
                                "sky" | "lco" => 1,
                                "detail" | "cdt" | "dt" | "mco" => 2,
                                "mc" => 7,
                                "as" => 8,
                                "sm" | "smdi" => 9,
                                "dtsmdi" => 10,
                                "mask" => 11,
                                _ => 0,
                            }
                        })
                    })
            },
            mipmaps: paa
                .maps()
                .iter()
                .map(|m| MipMap {
                    width: m.width(),
                    height: m.height(),
                    pax_format: u8::try_from(m.format().as_u32()).expect("MipMap format too large"),
                    data_offset: u32::try_from(m.offset()).expect("MipMap offset too large"),
                })
                .collect(),
            size_of_pax_file: u32::try_from(
                std::fs::metadata(path)
                    .expect("Failed to get PAA file metadata")
                    .len(),
            )
            .map_err(|_| std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "PAA file size exceeds u32 range"
            ))?,
        })
    }
}

// MipMap
// {
//  ushort width,height;            //as per same mipmap in the pax
//  ushort Always0;
//  byte   pax_format;              //same value as above
//  byte   Always3;
//  ulong  dataOffset;              //OFFSTAGG in PAx file. position of mipmap data in pax file
// };
#[derive(Debug, PartialEq, Eq)]
pub struct MipMap {
    width: u16,
    height: u16,
    pax_format: u8,
    data_offset: u32,
}

impl MipMap {
    /// Read the `MipMap` from the given input
    ///
    /// # Errors
    /// [`std::io::Error`] if the input is not readable, or the `MipMap` is invalid
    pub fn read<I: Seek + Read>(mut input: I) -> Result<Self, Error> {
        let width = input.read_u16::<LittleEndian>()?;
        let height = input.read_u16::<LittleEndian>()?;
        let _always_0 = input.read_u16::<LittleEndian>()?;
        let pax_format = input.read_u8()?;
        let _always_3 = input.read_u8()?;
        let data_offset = input.read_u32::<LittleEndian>()?;

        Ok(Self {
            width,
            height,
            pax_format,
            data_offset,
        })
    }

    /// Write the `MipMap` to the given output
    ///
    /// # Errors
    /// [`std::io::Error`] if the output is not writable
    pub fn write(mipmap: &Self, output: &mut impl Write) -> Result<(), Error> {
        output.write_u16::<LittleEndian>(mipmap.width)?;
        output.write_u16::<LittleEndian>(mipmap.height)?;
        output.write_u16::<LittleEndian>(0)?; // Always0
        output.write_u8(mipmap.pax_format)?;
        output.write_u8(3)?; // Always3
        output.write_u32::<LittleEndian>(mipmap.data_offset)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    #[test]
    fn read_and_write() {
        let mikero = include_bytes!("../../tests/existing.bin");
        let mut cursor = std::io::Cursor::new(&mikero[..]);
        let header = super::Headers::read(&mut cursor).expect("Failed to read Headers");
        let mut output = Vec::new();
        super::Headers::write(&header, &mut output).expect("Failed to write Headers");
        assert_eq!(
            &output[..],
            &mikero[..],
            "Written Headers do not match original"
        );
    }

    #[test]
    fn create() {
        let mikero = {
            let mikero = include_bytes!("../../tests/existing.bin");
            let mut cursor = std::io::Cursor::new(&mikero[..]);
            super::Headers::read(&mut cursor).expect("Failed to read Headers")
        };
        let mut headers = Vec::new();
        let root = std::path::PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"),
        )
        .join("tests");
        for path in ["dxt1.paa", "dxt5.paa"] {
            let path = root.join(path);
            headers.push(
                super::TextureHeader::from_file(&root, &path)
                    .expect("Failed to create TextureHeader from PAA file"),
            );
        }
        let header = super::Headers { textures: headers };
        assert_eq!(header, mikero, "Written Headers do not match original");
    }
}
