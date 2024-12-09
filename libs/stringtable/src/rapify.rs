use crate::{Key, Project, ALL_LANGUAGES};
use hemtt_workspace::WorkspacePath;
use tracing::trace;

#[derive(Default, Debug)]
pub struct XmlbLayout {
    //  4 byte numbers, little end?, nul term strings
    header: Vec<u8>,            // Version
    languages: Vec<u8>,         // Language Count, [Languages]
    offsets: Vec<u8>,           // Offset Count, [Offsets]
    keys: Vec<u8>,              // Key Count, [Keys]
    translations: Vec<Vec<u8>>, // [Translation Count, [Translations], ...] (size may be less than lang count)
}

#[derive(Clone, Debug)]
struct Translation {
    phrases: Vec<String>,
    have_unique: bool,
}

/// # Panics
pub fn convert_stringtable(project: &Project, xml_path: &WorkspacePath) {
    let result = rapify(project);

    if result.is_some() {
        // Create stringtable.bin
        let xmlb_path = xml_path.with_extension("bin").expect("vfs error");
        let mut xmlb_file = xmlb_path.create_file().expect("vfs error");

        // Remove Original stringtable.xml
        xml_path.vfs().remove_file().expect("vfs error");

        // Write data to virtual file
        let data = result.expect("data struct valid");
        xmlb_file.write_all(&data.header).expect("IO Error");
        xmlb_file.write_all(&data.languages).expect("IO Error");
        xmlb_file.write_all(&data.offsets).expect("IO Error");
        xmlb_file.write_all(&data.keys).expect("IO Error");
        for translation_buffer in &data.translations {
            xmlb_file.write_all(translation_buffer).expect("IO Error");
        }
        trace!(
            "binned stringtable{:?} [Unique {}]",
            xmlb_path,
            data.translations.len()
        );
    } else {
        trace!("skpping binerization of stringtable{:?}", xml_path);
    }
}

/// Write string with null-termination
fn write_string(buffer: &mut Vec<u8>, input: &str) {
    buffer.extend(input.as_bytes());
    buffer.push(0);
}
fn write_int(buffer: &mut Vec<u8>, input: i32) {
    buffer.extend(&input.to_le_bytes());
}

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
    write_int(&mut data.header, 1_481_460_802); // aka XMLB in LE

    // Languages
    write_int(
        &mut data.languages,
        i32::try_from(ALL_LANGUAGES.len()).expect("overflow"),
    );
    for language in ALL_LANGUAGES {
        write_string(&mut data.languages, language);
    }

    // Keys
    write_int(
        &mut data.keys,
        i32::try_from(all_keys.len()).expect("overflow"),
    );
    for key in &all_keys {
        write_string(&mut data.keys, key);
    }

    // Offset
    let offset_size = 4 + 4 * ALL_LANGUAGES.len();
    let mut rolling_offset =
        data.header.len() + data.languages.len() + offset_size + data.keys.len();
    write_int(
        &mut data.offsets,
        i32::try_from(ALL_LANGUAGES.len()).expect("overflow"),
    );

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
            write_int(
                &mut translation_buffer,
                i32::try_from(translation.phrases.len()).expect("overflow"),
            );
            for phrase in &translation.phrases {
                write_string(&mut translation_buffer, &phrase);
            }
            rolling_offset += translation_buffer.len();
            data.translations.push(translation_buffer);
            offset_start
        } else {
            // no unique translations, just use first offset (english)
            first_offset
        };

        write_int(&mut data.offsets, i32::try_from(offset).expect("overflow"));
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
