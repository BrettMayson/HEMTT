use std::collections::BTreeMap;

use quick_xml::escape::minimal_escape;
use serde::{Deserialize, Serialize, Serializer};

fn min_escape<S>(s: &String, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ser.serialize_str(&minimal_escape(s))
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct Language {
    // inner text element of the language tag, default allows the string to be empty
    #[serde(default, rename = "$text", serialize_with = "min_escape")]
    content: String,
    #[serde(flatten)]
    attributes: BTreeMap<String, String>,
}

macro_rules! define_key {
    ($( $field:ident => ($lower:literal, $upper:literal, $pascalAlt:literal, $display:literal, $setter:ident) ),* $(,)?) => {
        #[derive(Clone, Debug, Default, Deserialize, Serialize)]
        #[serde(rename_all = "PascalCase")]
        pub struct Key {
            #[serde(rename = "@ID")]
            id: String,
            #[serde(rename = "@allow-empty", skip_serializing_if = "Option::is_none")]
            allow_empty: Option<bool>,
            $(
                #[serde(skip_serializing_if = "Option::is_none")]
                #[serde(alias = $lower, alias = $upper, alias = $pascalAlt)]
                $field: Option<Language>,
            )*
            #[serde(flatten, skip_serializing)]
            #[allow(clippy::zero_sized_map_values)]
            unknown_languages: std::collections::HashMap<String, serde::de::IgnoredAny>,
        }

        impl Key {
            #[must_use]
            /// Create a new Key with the given ID
            pub fn new(id: String) -> Self {
                Self { id, ..Default::default() }
            }

            #[must_use]
            pub fn id(&self) -> &str {
                &self.id
            }

            $(
                #[must_use]
                pub fn $field(&self) -> Option<&str> {
                    self.$field.as_ref().map(|l| &l.content as &str)
                }
            )*

            /// Set the value for a specific language.
            ///
            /// # Panics
            /// If the language is unknown or not supported.
            pub fn set(&mut self, language: &str, value: String) {
                match language.to_lowercase().as_str() {
                    $( $lower => self.$setter = Some(Language {content: value, ..Default::default() }), )*
                    _ => panic!("Unknown language: {language}"),
                }
            }

            #[must_use]
            pub fn unknown_languages(&self) -> Vec<String> {
                self.unknown_languages.keys().filter(|k| !k.starts_with('@')).cloned().collect()
            }

            #[must_use]
            /// Get all language values as a list of tuples (language, value)
            pub fn as_list(&self) -> Vec<(&str, Option<&str>)> {
                vec![$( ($display, self.$field()), )*]
            }

            #[must_use]
            pub fn allow_empty(&self) -> bool {
                self.allow_empty.unwrap_or(false)
            }
        }
    };
}

// $field:ident => ($lower:literal, $upper:literal, $pascalAlt:literal, $display:literal, $setter:ident)
macro_rules! ALL_LANGUAGES {
    ($callback:ident) => {
        $callback! {
            original => ("original", "ORIGINAL", "Original", "Original", english), // set maps to english
            english => ("english", "ENGLISH", "English","English", english),
            czech => ("czech", "CZECH", "Czech", "Czech", czech),
            french => ("french", "FRENCH", "French", "French", french),
            spanish => ("spanish", "SPANISH", "Spanish", "Spanish", spanish),
            italian => ("italian", "ITALIAN", "Italian", "Italian", italian),
            polish => ("polish", "POLISH", "Polish", "Polish", polish),
            portuguese => ("portuguese", "PORTUGUESE", "Portuguese", "Portuguese", portuguese),
            russian => ("russian", "RUSSIAN", "Russian", "Russian", russian),
            german => ("german", "GERMAN", "German", "German", german),
            korean => ("korean", "KOREAN", "Korean", "Korean", korean),
            japanese => ("japanese", "JAPANESE", "Japanese", "Japanese", japanese),
            chinese => ("chinese", "CHINESE", "Chinese", "Chinese", chinese),
            chinesesimp => ("chinesesimp", "CHINESESIMP", "ChineseSimp","ChineseSimp", chinesesimp),
            turkish => ("turkish", "TURKISH", "Turkish", "Turkish", turkish),
            swedish => ("swedish", "SWEDISH", "Swedish", "Swedish", swedish),
            slovak => ("slovak", "SLOVAK", "Slovak", "Slovak", slovak),
            serbocroatian => ("serbocroatian", "SERBOCROATIAN", "SerboCroatian", "Serbocroatian", serbocroatian),
            norwegian => ("norwegian", "NORWEGIAN", "Norwegian", "Norwegian", norwegian),
            icelandic => ("icelandic", "ICELANDIC", "Icelandic", "Icelandic", icelandic),
            hungarian => ("hungarian", "HUNGARIAN", "Hungarian", "Hungarian", hungarian),
            greek => ("greek", "GREEK", "Greek", "Greek", greek),
            finnish => ("finnish", "FINNISH", "Finnish", "Finnish", finnish),
            dutch => ("dutch", "DUTCH", "Dutch", "Dutch", dutch),
            ukrainian => ("ukrainian", "UKRAINIAN", "Ukrainian", "Ukrainian", ukrainian),
            danish => ("danish", "DANISH", "Danish", "Danish", danish),
            bulgarian => ("bulgarian", "BULGARIAN", "Bulgarian", "Bulgarian", bulgarian),
            latin => ("latin", "LATIN", "Latin", "Latin", latin),
        }
    };
}

ALL_LANGUAGES!(define_key);
