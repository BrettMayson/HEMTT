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
    #[serde(default, rename = "$value", serialize_with = "min_escape")]
    content: String,
    #[serde(rename = "@ai-translated", skip_serializing_if = "Option::is_none")]
    ai_translated: Option<bool>,
    #[serde(rename = "@needs-review", skip_serializing_if = "Option::is_none")]
    needs_review: Option<bool>,
}

macro_rules! define_key {
    ($( $field:ident => ($lower:literal, $upper:literal, $display:literal, $setter:ident) ),* $(,)?) => {
        #[derive(Clone, Debug, Default, Deserialize, Serialize)]
        #[serde(rename_all = "PascalCase")]
        pub struct Key {
            #[serde(rename = "@ID")]
            id: String,
            #[serde(rename = "@allow-empty", skip_serializing_if = "Option::is_none")]
            allow_empty: Option<bool>,
            $(
                #[serde(skip_serializing_if = "Option::is_none")]
                #[serde(alias = $lower, alias = $upper)]
                $field: Option<Language>,
            )*
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

macro_rules! ALL_LANGUAGES {
    ($callback:ident) => {
        $callback! {
            original => ("original", "ORIGINAL", "Original", english), // set maps to english
            english => ("english", "ENGLISH", "English", english),
            czech => ("czech", "CZECH", "Czech", czech),
            french => ("french", "FRENCH", "French", french),
            spanish => ("spanish", "SPANISH", "Spanish", spanish),
            italian => ("italian", "ITALIAN", "Italian", italian),
            polish => ("polish", "POLISH", "Polish", polish),
            portuguese => ("portuguese", "PORTUGUESE", "Portuguese", portuguese),
            russian => ("russian", "RUSSIAN", "Russian", russian),
            german => ("german", "GERMAN", "German", german),
            korean => ("korean", "KOREAN", "Korean", korean),
            japanese => ("japanese", "JAPANESE", "Japanese", japanese),
            chinese => ("chinese", "CHINESE", "Chinese", chinese),
            chinesesimp => ("chinesesimp", "CHINESESIMP", "ChineseSimp", chinesesimp),
            turkish => ("turkish", "TURKISH", "Turkish", turkish),
            swedish => ("swedish", "SWEDISH", "Swedish", swedish),
            slovak => ("slovak", "SLOVAK", "Slovak", slovak),
            serbocroatian => ("serbocroatian", "SERBOCROATIAN", "Serbocroatian", serbocroatian),
            norwegian => ("norwegian", "NORWEGIAN", "Norwegian", norwegian),
            icelandic => ("icelandic", "ICELANDIC", "Icelandic", icelandic),
            hungarian => ("hungarian", "HUNGARIAN", "Hungarian", hungarian),
            greek => ("greek", "GREEK", "Greek", greek),
            finnish => ("finnish", "FINNISH", "Finnish", finnish),
            dutch => ("dutch", "DUTCH", "Dutch", dutch),
            ukrainian => ("ukrainian", "UKRAINIAN", "Ukrainian", ukrainian),
            danish => ("danish", "DANISH", "Danish", danish),
            bulgarian => ("bulgarian", "BULGARIAN", "Bulgarian", bulgarian),
            latin => ("latin", "LATIN", "Latin", latin),
        }
    };
}

ALL_LANGUAGES!(define_key);
