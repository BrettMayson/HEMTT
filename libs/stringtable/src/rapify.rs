use std::io::Write;

use crate::{ALL_LANGUAGES, Key, Project};
use byteorder::{LittleEndian, WriteBytesExt};
use hemtt_common::io::WriteExt;
use tracing::{trace, warn};

#[derive(Default, Debug)]
pub struct XmlbLayout {
    //  4 byte numbers, little end?, nul term strings
    pub header: Vec<u8>,            // Version
    pub languages: Vec<u8>,         // Language Count, [Languages]
    pub offsets: Vec<u8>,           // Offset Count, [Offsets]
    pub keys: Vec<u8>,              // Key Count, [Keys]
    pub translations: Vec<Vec<u8>>, // [Translation Count, [Translations], ...] (size may be less than lang count)
}

impl XmlbLayout {
    /// Writes the XMLB layout to a writer
    ///
    /// # Errors
    /// [`std::io::Error`] if writing to the writer fails
    pub fn write(&self, writer: &mut dyn Write) -> std::io::Result<()> {
        writer.write_all(&self.header)?;
        writer.write_all(&self.languages)?;
        writer.write_all(&self.offsets)?;
        writer.write_all(&self.keys)?;
        for translation_buffer in &self.translations {
            writer.write_all(translation_buffer)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Translation {
    phrases: Vec<String>,
    have_unique: bool,
}

/// Converts a stringtable.xml to a stringtable.bin
///
/// # Panics
/// If the files can't be read or written from the vfs
pub fn convert_stringtable(project: &Project) {
    let result = rapify(project);

    if result.is_some() {
        // Create stringtable.bin
        let xmlb_path = project.path().with_extension("bin").expect("vfs error");
        let mut xmlb_file = xmlb_path.create_file().expect("vfs error");

        // Remove Original stringtable.xml
        project.path().vfs().remove_file().expect("vfs error");

        // Write data to virtual file
        let data = result.expect("data struct valid");
        data.write(&mut xmlb_file)
            .expect("failed to write XMLB layout");
        trace!(
            "binned stringtable{:?} [Unique {}]",
            xmlb_path,
            data.translations.len()
        );
    } else {
        trace!("skpping binerization of stringtable{:?}", project.path());
    }
}

#[must_use]
/// # Panics
pub fn rapify(project: &Project) -> Option<XmlbLayout> {
    let mut data: XmlbLayout = XmlbLayout::default();

    // Restructure translations: flat for each language
    let mut all_keys: Vec<String> = Vec::with_capacity(20);
    let mut all_translations: Vec<Translation> = vec![
        Translation {
            phrases: Vec::with_capacity(20),
            have_unique: false
        };
        ALL_LANGUAGES.len()
    ];

    for package in project.packages() {
        for package_inner in package.containers() {
            for key in package_inner.keys() {
                all_keys.push(key.id().into());
                if !get_translations(key, &mut all_translations) {
                    return None; // stop if we can't get some kind of translation
                }
            }
        }
        for key in package.keys() {
            all_keys.push(key.id().into());
            if !get_translations(key, &mut all_translations) {
                return None; // stop if we can't get some kind of translation
            }
        }
    }

    // Header
    data.header
        .write_all(b"BLMX") // XMLB in LE
        .expect("failed to write header magic");

    // Languages
    data.languages
        .write_i32::<LittleEndian>(i32::try_from(ALL_LANGUAGES.len()).expect("overflow"))
        .expect("failed to write language count");
    for language in ALL_LANGUAGES {
        let _ = data.languages.write_cstring(language);
    }

    // Keys
    data.keys
        .write_i32::<LittleEndian>(i32::try_from(all_keys.len()).expect("overflow"))
        .expect("failed to write key count");
    for key in &all_keys {
        let _ = data.keys.write_cstring(key);
    }

    // Offset
    let offset_size = 4 + 4 * ALL_LANGUAGES.len();
    let mut rolling_offset =
        data.header.len() + data.languages.len() + offset_size + data.keys.len();
    data.offsets
        .write_i32::<LittleEndian>(i32::try_from(ALL_LANGUAGES.len()).expect("overflow"))
        .expect("failed to write offset count");

    all_translations[0].have_unique = true; // Always write first set (english)
    let first_offset = rolling_offset;

    // Languages and their offsets
    for translation in all_translations {
        debug_assert_eq!(translation.phrases.len(), all_keys.len());

        let offset = if translation.have_unique {
            // we have some unique tranlation, write and use it's offset
            let offset_start = rolling_offset;
            let mut translation_buffer: Vec<u8> =
                Vec::with_capacity(32 * translation.phrases.len());
            translation_buffer
                .write_i32::<LittleEndian>(
                    i32::try_from(translation.phrases.len()).expect("overflow"),
                )
                .expect("failed to write translation count");
            for phrase in &translation.phrases {
                let unescaped = quick_xml::escape::unescape(phrase.as_str());
                if unescaped.is_err() {
                    warn!("failed to unescape stringtable entry [{}]", phrase);
                    return None;
                }
                let _ = translation_buffer.write_cstring(&*unescaped.unwrap_or_default());
            }
            rolling_offset += translation_buffer.len();
            data.translations.push(translation_buffer);
            offset_start
        } else {
            // no unique translations, just use first offset (english)
            first_offset
        };

        data.offsets
            .write_i32::<LittleEndian>(i32::try_from(offset).expect("overflow"))
            .expect("failed to write offset");
    }
    debug_assert_eq!(offset_size, data.offsets.len());

    Some(data)
}

fn get_translations(key: &Key, languages: &mut [Translation]) -> bool {
    let tranlations = [
        key.english(),
        key.czech(),
        key.french(),
        key.spanish(),
        key.italian(),
        key.polish(),
        key.portuguese(),
        key.russian(),
        key.german(),
        key.korean(),
        key.japanese(),
        key.chinese(),
        key.chinesesimp(),
        key.turkish(),
        key.swedish(),
        key.slovak(),
        key.serbocroatian(),
        key.norwegian(),
        key.icelandic(),
        key.hungarian(),
        key.greek(),
        key.finnish(),
        key.dutch(),
        key.ukrainian(),
        key.danish(),
    ];
    debug_assert_eq!(tranlations.len(), ALL_LANGUAGES.len()); // order needs to be synced // Todo: meta programing?

    for (index, result) in tranlations.into_iter().enumerate() {
        if let Some(native) = result {
            languages[index].have_unique = true;
            languages[index].phrases.push(native.into());
        } else if let Some(original) = key.original() {
            languages[index].phrases.push(original.into());
        } else if let Some(english) = key.english() {
            languages[index].phrases.push(english.into());
        } else {
            // If we don't have some kind of default value to use, we should just not do the conversion
            return false;
        }
    }

    true
}
