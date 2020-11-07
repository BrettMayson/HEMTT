//! Functions for rapifying and derapifying Arma configs

use std::io::{BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write};
use std::iter::Sum;
use std::path::PathBuf;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::preprocess::*;
use crate::*;
use hemtt_io::*;

use crate::error::ConfigParseError;
use crate::HEMTTError;

pub mod grammar {
    #![allow(missing_docs)]
    #![allow(deprecated)]
    include!(concat!(env!("OUT_DIR"), "/config.rs"));
}

/// Config
///
/// # Examples
///
/// ```
/// # use hemtt::Config;
/// let input = String::from("
/// #define bar 42
///
/// foo = bar;
/// ");
///
/// let config = Config::from_string(input, None, &Vec::new(), |path| std::fs::read_to_string(&path).map_err(|e| e.into())).expect("Failed to parse config");
///
/// assert_eq!("foo = 42;\n", config.to_string().unwrap());
/// assert_eq!(b"\0raP", &config.to_cursor().unwrap().into_inner()[..4]);
/// ```
#[derive(Debug)]
pub struct Config {
    pub root_body: ConfigClass,
}

/// Config class
#[derive(Debug)]
pub struct ConfigClass {
    pub parent: String,
    pub is_external: bool,
    pub is_deletion: bool,
    pub entries: Option<Vec<(String, ConfigEntry)>>,
}

/// Config entry
#[derive(Debug)]
pub enum ConfigEntry {
    /// String entry
    StringEntry(String),
    /// Float entry
    FloatEntry(f32),
    /// Int entry
    IntEntry(i32),
    /// Array entry
    ArrayEntry(ConfigArray),
    /// Class entry
    ClassEntry(ConfigClass),
}

/// Config array
#[derive(Debug)]
pub struct ConfigArray {
    pub is_expansion: bool,
    pub elements: Vec<ConfigArrayElement>,
}

/// Config array element
#[derive(Debug)]
pub enum ConfigArrayElement {
    /// String element
    StringElement(String),
    /// Float element
    FloatElement(f32),
    /// Int element
    IntElement(i32),
    /// Array element
    ArrayElement(ConfigArray),
}

impl ConfigArrayElement {
    pub fn rapified_length(&self) -> usize {
        match self {
            ConfigArrayElement::StringElement(s) => s.len() + 2,
            ConfigArrayElement::FloatElement(_f) => 5,
            ConfigArrayElement::IntElement(_i) => 5,
            ConfigArrayElement::ArrayElement(a) => {
                1 + compressed_int_len(a.elements.len() as u32)
                    + usize::sum(a.elements.iter().map(|e| e.rapified_length()))
            }
        }
    }
}

impl ConfigArray {
    pub fn write<O: Write>(&self, output: &mut O) -> Result<(), HEMTTError> {
        output.write_all(b"{")?;
        for (key, value) in self.elements.iter().enumerate() {
            match value {
                ConfigArrayElement::ArrayElement(ref a) => {
                    a.write(output)?;
                }
                ConfigArrayElement::StringElement(s) => {
                    output.write_all(
                        format!(
                            "\"{}\"",
                            s.replace("\r", "\\r")
                                .replace("\n", "\\n")
                                .replace("\"", "\"\"")
                        )
                        .as_bytes(),
                    )?;
                }
                ConfigArrayElement::FloatElement(f) => {
                    output.write_all(format!("{:?}", f).as_bytes())?;
                }
                ConfigArrayElement::IntElement(i) => {
                    output.write_all(format!("{}", i).as_bytes())?;
                }
            }
            if key < self.elements.len() - 1 {
                output.write_all(b", ")?;
            }
        }
        output.write_all(b"}")?;
        Ok(())
    }

    pub fn write_rapified<O: Write>(&self, output: &mut O) -> Result<usize, HEMTTError> {
        let mut written = output.write_compressed_int(self.elements.len() as u32)?;

        for element in &self.elements {
            match element {
                ConfigArrayElement::StringElement(s) => {
                    output.write_all(&[0])?;
                    output.write_cstring(s)?;
                    written += s.len() + 2;
                }
                ConfigArrayElement::FloatElement(f) => {
                    output.write_all(&[1])?;
                    output.write_f32::<LittleEndian>(*f)?;
                    written += 5;
                }
                ConfigArrayElement::IntElement(i) => {
                    output.write_all(&[2])?;
                    output.write_i32::<LittleEndian>(*i)?;
                    written += 5;
                }
                ConfigArrayElement::ArrayElement(a) => {
                    output.write_all(&[3])?;
                    written += 1 + a.write_rapified(output)?;
                }
            }
        }

        Ok(written)
    }

    pub fn read_rapified<I: Read + Seek>(input: &mut I) -> Result<ConfigArray, HEMTTError> {
        let num_elements: u32 = input.read_compressed_int()?;
        let mut elements: Vec<ConfigArrayElement> = Vec::with_capacity(num_elements as usize);

        for _i in 0..num_elements {
            let element_type: u8 = input.bytes().next().unwrap()?;

            if element_type == 0 {
                elements.push(ConfigArrayElement::StringElement(input.read_cstring()?));
            } else if element_type == 1 {
                elements.push(ConfigArrayElement::FloatElement(
                    input.read_f32::<LittleEndian>()?,
                ));
            } else if element_type == 2 {
                elements.push(ConfigArrayElement::IntElement(
                    input.read_i32::<LittleEndian>()?,
                ));
            } else if element_type == 3 {
                elements.push(ConfigArrayElement::ArrayElement(
                    ConfigArray::read_rapified(input)?,
                ));
            } else {
                return Err(aerror!("Unrecognized array element type: {}", element_type));
            }
        }

        Ok(ConfigArray {
            is_expansion: false,
            elements,
        })
    }
}

impl ConfigEntry {
    // without the name
    pub fn rapified_length(&self) -> usize {
        match self {
            ConfigEntry::StringEntry(s) => s.len() + 3,
            ConfigEntry::FloatEntry(_f) => 6,
            ConfigEntry::IntEntry(_i) => 6,
            ConfigEntry::ArrayEntry(a) => {
                let len = 1
                    + compressed_int_len(a.elements.len() as u32)
                    + usize::sum(a.elements.iter().map(|e| e.rapified_length()));
                if a.is_expansion {
                    len + 4
                } else {
                    len
                }
            }
            ConfigEntry::ClassEntry(c) => {
                if c.is_external || c.is_deletion {
                    1
                } else {
                    5
                }
            }
        }
    }
}

impl ConfigClass {
    pub fn write<O: Write>(&self, mut output: &mut O, level: i32) -> Result<(), HEMTTError> {
        match &self.entries {
            Some(entries) => {
                if level > 0 && !entries.is_empty() {
                    output.write_all(b"\n")?;
                }
                for (key, value) in entries {
                    output.write_all(String::from("    ").repeat(level as usize).as_bytes())?;

                    match value {
                        ConfigEntry::ClassEntry(ref c) => {
                            if c.is_deletion {
                                output.write_all(format!("delete {};\n", key).as_bytes())?;
                            } else if c.is_external {
                                output.write_all(format!("class {};\n", key).as_bytes())?;
                            } else {
                                let parent = if c.parent == "" {
                                    String::from("")
                                } else {
                                    format!(": {}", c.parent)
                                };
                                match &c.entries {
                                    Some(entries) => {
                                        if !entries.is_empty() {
                                            output.write_all(
                                                format!("class {}{} {{", key, parent).as_bytes(),
                                            )?;
                                            c.write(output, level + 1)?;
                                            output.write_all(
                                                String::from("    ")
                                                    .repeat(level as usize)
                                                    .as_bytes(),
                                            )?;
                                            output.write_all(b"};\n")?;
                                        } else {
                                            output.write_all(
                                                format!("class {}{} {{}};\n", key, parent)
                                                    .as_bytes(),
                                            )?;
                                        }
                                    }
                                    None => {
                                        output.write_all(
                                            format!("class {}{} {{}};\n", key, parent).as_bytes(),
                                        )?;
                                    }
                                }
                            }
                        }
                        ConfigEntry::StringEntry(s) => {
                            output.write_all(
                                format!(
                                    "{} = \"{}\";\n",
                                    key,
                                    s.replace("\r", "\\r")
                                        .replace("\n", "\\n")
                                        .replace("\"", "\"\"")
                                )
                                .as_bytes(),
                            )?;
                        }
                        ConfigEntry::FloatEntry(f) => {
                            output.write_all(format!("{} = {:?};\n", key, f).as_bytes())?;
                        }
                        ConfigEntry::IntEntry(i) => {
                            output.write_all(format!("{} = {};\n", key, i).as_bytes())?;
                        }
                        ConfigEntry::ArrayEntry(ref a) => {
                            if a.is_expansion {
                                output.write_all(format!("{}[] += ", key).as_bytes())?;
                            } else {
                                output.write_all(format!("{}[] = ", key).as_bytes())?;
                            }
                            a.write(&mut output)?;
                            output.write_all(b";\n")?;
                        }
                    }
                }
            }
            None => {}
        }

        Ok(())
    }

    pub fn rapified_length(&self) -> usize {
        match &self.entries {
            Some(entries) => {
                self.parent.len()
                    + 1
                    + compressed_int_len(entries.len() as u32)
                    + usize::sum(entries.iter().map(|(k, v)| {
                        k.len()
                            + 1
                            + v.rapified_length()
                            + match v {
                                ConfigEntry::ClassEntry(c) => c.rapified_length(),
                                _ => 0,
                            }
                    }))
            }
            None => 0,
        }
    }

    pub fn write_rapified<O: Write>(
        &self,
        output: &mut O,
        offset: usize,
    ) -> Result<usize, HEMTTError> {
        let mut written = 0;

        match &self.entries {
            Some(entries) => {
                output.write_cstring(&self.parent)?;
                written += self.parent.len() + 1;

                written += output.write_compressed_int(entries.len() as u32)?;

                let entries_len = usize::sum(
                    entries
                        .iter()
                        .map(|(k, v)| k.len() + 1 + v.rapified_length()),
                );
                let mut class_offset = offset + written + entries_len;
                let mut class_bodies: Vec<Cursor<Box<[u8]>>> = Vec::new();
                let pre_entries = written;

                for (name, entry) in entries {
                    let pre_write = written;
                    match entry {
                        ConfigEntry::StringEntry(s) => {
                            output.write_all(&[1, 0])?;
                            output.write_cstring(name)?;
                            output.write_cstring(s)?;
                            written += name.len() + s.len() + 4;
                        }
                        ConfigEntry::FloatEntry(f) => {
                            output.write_all(&[1, 1])?;
                            output.write_cstring(name)?;
                            output.write_f32::<LittleEndian>(*f)?;
                            written += name.len() + 7;
                        }
                        ConfigEntry::IntEntry(i) => {
                            output.write_all(&[1, 2])?;
                            output.write_cstring(name)?;
                            output.write_i32::<LittleEndian>(*i)?;
                            written += name.len() + 7;
                        }
                        ConfigEntry::ArrayEntry(a) => {
                            output.write_all(if a.is_expansion { &[5] } else { &[2] })?;
                            if a.is_expansion {
                                output.write_all(&[1, 0, 0, 0])?;
                                written += 4;
                            }
                            output.write_cstring(name)?;
                            written += name.len() + 2 + a.write_rapified(output)?;
                        }
                        ConfigEntry::ClassEntry(c) => {
                            if c.is_external || c.is_deletion {
                                output.write_all(if c.is_deletion { &[4] } else { &[3] })?;
                                output.write_cstring(name)?;
                                written += name.len() + 2;
                            } else {
                                output.write_all(&[0])?;
                                output.write_cstring(name)?;
                                output.write_u32::<LittleEndian>(class_offset as u32)?;
                                written += name.len() + 6;

                                let buffer: Box<[u8]> =
                                    vec![0; c.rapified_length()].into_boxed_slice();
                                let mut cursor: Cursor<Box<[u8]>> = Cursor::new(buffer);
                                class_offset += c.write_rapified(&mut cursor, class_offset)?;
                                class_bodies.push(cursor);
                            }
                        }
                    }
                    assert_eq!(
                        written - pre_write,
                        entry.rapified_length() + name.len() + 1
                    );
                }

                assert_eq!(written - pre_entries, entries_len);

                for cursor in class_bodies {
                    output.write_all(cursor.get_ref())?;
                    written += cursor.get_ref().len();
                }
            }
            None => unreachable!(),
        }

        Ok(written)
    }

    pub fn read_rapified<I: Read + Seek>(
        input: &mut I,
        level: u32,
    ) -> Result<ConfigClass, HEMTTError> {
        let mut fp = 0;
        if level == 0 {
            input.seek(SeekFrom::Start(16))?;
        } else {
            let classbody_fp: u32 = input.read_u32::<LittleEndian>()?;

            fp = input.seek(SeekFrom::Current(0))?;
            input.seek(SeekFrom::Start(classbody_fp.into()))?;
        }

        let parent = input.read_cstring()?;
        let num_entries: u32 = input.read_compressed_int()?;
        let mut entries: Vec<(String, ConfigEntry)> = Vec::with_capacity(num_entries as usize);

        for _i in 0..num_entries {
            let entry_type: u8 = input.bytes().next().unwrap()?;

            if entry_type == 0 {
                let name = input.read_cstring()?;

                let class_entry = ConfigClass::read_rapified(input, level + 1)?;
                entries.push((name, ConfigEntry::ClassEntry(class_entry)));
            } else if entry_type == 1 {
                let subtype: u8 = input.bytes().next().unwrap()?;
                let name = input.read_cstring()?;

                if subtype == 0 {
                    entries.push((name, ConfigEntry::StringEntry(input.read_cstring()?)));
                } else if subtype == 1 {
                    entries.push((
                        name,
                        ConfigEntry::FloatEntry(input.read_f32::<LittleEndian>()?),
                    ));
                } else if subtype == 2 {
                    entries.push((
                        name,
                        ConfigEntry::IntEntry(input.read_i32::<LittleEndian>()?),
                    ));
                } else {
                    return Err(aerror!("Unrecognized variable entry subtype: {}.", subtype));
                }
            } else if entry_type == 2 || entry_type == 5 {
                if entry_type == 5 {
                    input.seek(SeekFrom::Current(4))?;
                }

                let name = input.read_cstring()?;
                let mut array = ConfigArray::read_rapified(input)?;
                array.is_expansion = entry_type == 5;

                entries.push((name.clone(), ConfigEntry::ArrayEntry(array)));
            } else if entry_type == 3 || entry_type == 4 {
                let name = input.read_cstring()?;
                let class_entry = ConfigClass {
                    parent: String::from(""),
                    is_external: entry_type == 3,
                    is_deletion: entry_type == 5,
                    entries: None,
                };

                entries.push((name.clone(), ConfigEntry::ClassEntry(class_entry)));
            } else {
                return Err(aerror!("Unrecognized class entry type: {}.", entry_type));
            }
        }

        if level > 0 {
            input.seek(SeekFrom::Start(fp))?;
        }

        Ok(ConfigClass {
            parent,
            is_external: false,
            is_deletion: false,
            entries: Some(entries),
        })
    }
}

impl Config {
    /// Writes the config (unrapified) to the output.
    pub fn write<O: Write>(&self, output: &mut O) -> Result<(), HEMTTError> {
        self.root_body.write(output, 0)
    }

    /// Returns the unrapified config as a string.
    pub fn to_string(&self) -> Result<String, HEMTTError> {
        let buffer = Vec::new();
        let mut cursor: Cursor<Vec<u8>> = Cursor::new(buffer);
        self.write(&mut cursor)?;

        Ok(String::from_utf8(cursor.into_inner()).unwrap())
    }

    /// Writes the rapified config to the output.
    pub fn write_rapified<O: Write>(&self, output: &mut O) -> Result<(), HEMTTError> {
        let mut writer = BufWriter::new(output);

        writer.write_all(b"\0raP")?;
        writer.write_all(b"\0\0\0\0\x08\0\0\0")?; // always_0, always_8

        let buffer: Box<[u8]> = vec![0; self.root_body.rapified_length()].into_boxed_slice();
        let mut cursor: Cursor<Box<[u8]>> = Cursor::new(buffer);
        self.root_body.write_rapified(&mut cursor, 16)?;

        let enum_offset: u32 = 16 + cursor.get_ref().len() as u32;
        writer.write_u32::<LittleEndian>(enum_offset)?;

        writer.write_all(cursor.get_ref())?;

        writer.write_all(b"\0\0\0\0")?;

        Ok(())
    }

    /// Returns the rapified config as a `Cursor`.
    pub fn to_cursor(&self) -> Result<Cursor<Box<[u8]>>, HEMTTError> {
        let len = self.root_body.rapified_length() + 20;

        let buffer: Box<[u8]> = vec![0; len].into_boxed_slice();
        let mut cursor: Cursor<Box<[u8]>> = Cursor::new(buffer);
        self.write_rapified(&mut cursor)?;

        Ok(cursor)
    }

    /// Reads the unrapified config from input, preprocessing it.
    ///
    /// `path` is the path to the input if it is known and is used for relative includes and error
    /// messages. `includefolders` are the folders searched for absolute includes and should usually at
    /// least include the current working directory.
    pub fn read<I: Read, F>(
        input: &mut I,
        path: Option<PathBuf>,
        includefolders: &[PathBuf],
        fileread: F,
    ) -> Result<Config, HEMTTError>
    where
        F: Fn(&PathBuf) -> Result<String, HEMTTError>,
        F: Copy,
    {
        let mut buffer = String::new();
        input.read_to_string(&mut buffer)?;

        let (preprocessed, info) =
            preprocess(buffer.clone(), path.clone(), includefolders, fileread)?;

        let mut warnings: Vec<(usize, String, Option<&'static str>)> = Vec::new();

        let result = grammar::config(&preprocessed, &mut warnings).map_err(|source| {
            HEMTTError::Config(ConfigParseError {
                path: Some(
                    path.unwrap_or_else(PathBuf::new)
                        .to_string_lossy()
                        .to_string(),
                ),
                message: buffer,
                source,
            })
        })?;

        for w in warnings {
            // if !warning_suppressed(w.2) {
            let mut line = preprocessed[..w.0].chars().filter(|c| c == &'\n').count();
            let file = info.line_origins[std::cmp::min(line, info.line_origins.len()) - 1]
                .1
                .as_ref()
                .map(|p| p.to_str().unwrap().to_string());
            line =
                info.line_origins[std::cmp::min(line, info.line_origins.len()) - 1].0 as usize + 1;

            if let Some(f) = file {
                let clean = f
                    .trim_start_matches("\\\\?\\")
                    .trim_start_matches(&std::env::current_dir().unwrap().display().to_string());
                warn!("[{}:{}] {}", clean, line as u32, w.1);
            } else {
                warn!("[?:{}] {}", line as u32, w.1);
            }
            // } else {
            //     (None, None)
            // };
        }

        Ok(result)
    }

    /// Preprocesses and parses input string.
    ///
    /// `path` is the path to the input if it is known and is used for relative includes and error
    /// messages. `includefolders` are the folders searched for absolute includes and should usually at
    /// least include the current working directory.
    pub fn from_string<F>(
        input: String,
        path: Option<PathBuf>,
        includefolders: &[PathBuf],
        fileread: F,
    ) -> Result<Config, HEMTTError>
    where
        F: Fn(&PathBuf) -> Result<String, HEMTTError>,
        F: Copy,
    {
        let mut cursor = Cursor::new(input.into_bytes());
        Self::read(&mut cursor, path, includefolders, fileread)
    }

    /// Reads the rapified config from input.
    pub fn read_rapified<I: Read + Seek>(input: &mut I) -> Result<Config, HEMTTError> {
        let mut reader = BufReader::new(input);

        let mut buffer = [0; 4];
        reader.read_exact(&mut buffer)?;

        if &buffer != b"\0raP" {
            return Err(aerror!("File doesn't seem to be a rapified config."));
        }

        Ok(Config {
            root_body: ConfigClass::read_rapified(&mut reader, 0)?,
        })
    }
}
