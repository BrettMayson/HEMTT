use crate::{Key, Project};
use hemtt_workspace::WorkspacePath;
use tracing::warn;

static ALL_LANGUAGES: [&str; 14] = ["English", "Czech", "French", "Spanish", "Italian", "Polish", "Portuguese", "Russian", "German", "Korean", "Japanese", "Chinese", "Chinesesimp", "Turkish"];

#[derive(Default)]
struct XmlbLayout {
    //  4 byte numbers, little end?, nul term strings
    header: Vec<u8>,            // Version
    languages: Vec<u8>,         // Language Count, [Languages]
    offsets: Vec<u8>,           // Offset Count, [Offsets]
    keys: Vec<u8>,              // Key Count, [Keys]
    translations: Vec<Vec<u8>>, // [Translation Count, [Translations], ...]
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

        let data = result.expect("data struct valid");
        xmlb_file.write_all(&data.header).expect("IO Error");
        xmlb_file.write_all(&data.languages).expect("IO Error");
        xmlb_file.write_all(&data.offsets).expect("IO Error");
        xmlb_file.write_all(&data.keys).expect("IO Error");
        for translation_buffer in &data.translations {
            xmlb_file.write_all(translation_buffer).expect("IO Error");
        }
        println!("pass");
    } else {
        warn!("fail");
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

fn rapify(project: &Project) -> Option<XmlbLayout> {
    let mut data: XmlbLayout = XmlbLayout::default();

    // Restructure translations: flat for each language
    let mut all_keys: Vec<String> = Vec::new();
    let mut all_translations: Vec<Vec<String>> = vec![Vec::new(); ALL_LANGUAGES.len()];



    for package in project.packages() {
        for package_inner in package.containers() { // ugh
            for key in package_inner.keys() {
                all_keys.push(key.id().into());
                // Make sure we can translate everything
                if !get_translations(key, &mut all_translations) {
                    return None;
                }
            }
        }
        for key in package.keys() {
            all_keys.push(key.id().into());
            // Make sure we can translate everything
            if !get_translations(key, &mut all_translations) {
                return None;
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

    // compute offsets after finalizing languages

    // Keys
    write_int(
        &mut data.keys,
        i32::try_from(all_keys.len()).expect("overflow"),
    );
    for key in &all_keys {
        write_string(&mut data.keys, key);
    }

    // Languages
    for translations in all_translations {
        debug_assert_eq!(translations.len(), all_keys.len());
        let mut translation_buffer: Vec<u8> = Vec::new();
        write_int(
            &mut translation_buffer,
            i32::try_from(translations.len()).expect("overflow"),
        );
        for string in translations {
            write_string(&mut translation_buffer, &string);
        }
        data.translations.push(translation_buffer);
    }

    // Offsets
    let offset_size_estimate = 4 + 4 * ALL_LANGUAGES.len();
    let mut rolling_offset =
        data.header.len() + data.languages.len() + offset_size_estimate + data.keys.len();

    write_int(
        &mut data.offsets,
        i32::try_from(ALL_LANGUAGES.len()).expect("overflow"),
    );
    for translation_buffer in &data.translations {
        write_int(
            &mut data.offsets,
            i32::try_from(rolling_offset).expect("overflow"),
        );
        rolling_offset += translation_buffer.len();
    }

    debug_assert_eq!(offset_size_estimate, data.offsets.len());

    Some(data)
}

fn get_translations(key: &Key, all_translations: &mut [Vec<String>]) -> bool {
    let tranlations = [key.english(), key.czech(), key.french(), key.spanish(), key.italian(), key.polish(), key.portuguese(), key.russian(), key.german(), key.korean(), key.japanese(), key.chinese(), key.chinesesimp(), key.turkish()];
    debug_assert_eq!(tranlations.len(), ALL_LANGUAGES.len()); // needs to be synced

    for (index, result) in tranlations.into_iter().enumerate() {
        if let Some(native) = result {
            all_translations[index].push(native.into());
        } else if let Some(original) = key.original() {
            all_translations[index].push(original.into());
        } else if let Some(english) = key.english() {
            all_translations[index].push(english.into());
        } else {
            // If we don't have some kind of default value to use, we should just not do the conversion
            return false;
        }
    }

    true
}
