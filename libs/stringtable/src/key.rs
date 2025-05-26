use quick_xml::escape::minimal_escape;
use serde::{Deserialize, Serialize, Serializer};

#[allow(clippy::ref_option)] // `&Option<String>` is expected by serde
fn min_escape<S>(s: &Option<String>, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match s {
        Some(s) => ser.serialize_str(&minimal_escape(s)),
        None => ser.serialize_none(),
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Key {
    #[serde(rename = "@ID")]
    id: String,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "original", alias = "ORIGINAL")]
    original: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "english", alias = "ENGLISH")]
    english: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "czech", alias = "CZECH")]
    czech: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "french", alias = "FRENCH")]
    french: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "spanish", alias = "SPANISH")]
    spanish: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "italian", alias = "ITALIAN")]
    italian: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "polish", alias = "POLISH")]
    polish: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "portuguese", alias = "PORTUGUESE")]
    portuguese: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "russian", alias = "RUSSIAN")]
    russian: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "german", alias = "GERMAN")]
    german: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "korean", alias = "KOREAN")]
    korean: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "japanese", alias = "JAPANESE")]
    japanese: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "chinese", alias = "CHINESE")]
    chinese: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "chinesesimp", alias = "CHINESESIMP")]
    chinesesimp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "turkish", alias = "TURKISH")]
    turkish: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "swedish", alias = "SWEDISH")]
    swedish: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "slovak", alias = "SLOVAK")]
    slovak: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "serbocroatian", alias = "SERBOCROATIAN")]
    serbocroatian: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "norwegian", alias = "NORWEGIAN")]
    norwegian: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "icelandic", alias = "ICELANDIC")]
    icelandic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "hungarian", alias = "HUNGARIAN")]
    hungarian: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "greek", alias = "GREEK")]
    greek: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "finnish", alias = "FINNISH")]
    finnish: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "dutch", alias = "DUTCH")]
    dutch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "ukrainian", alias = "UKRAINIAN")]
    ukrainian: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "min_escape")]
    #[serde(alias = "danish", alias = "DANISH")]
    danish: Option<String>,
}

impl Key {
    #[must_use]
    /// Create a new Key with the given ID
    pub fn new(id: String) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn original(&self) -> Option<&str> {
        self.original.as_deref()
    }

    #[must_use]
    pub fn english(&self) -> Option<&str> {
        self.english.as_deref()
    }

    #[must_use]
    pub fn czech(&self) -> Option<&str> {
        self.czech.as_deref()
    }

    #[must_use]
    pub fn french(&self) -> Option<&str> {
        self.french.as_deref()
    }

    #[must_use]
    pub fn spanish(&self) -> Option<&str> {
        self.spanish.as_deref()
    }

    #[must_use]
    pub fn italian(&self) -> Option<&str> {
        self.italian.as_deref()
    }

    #[must_use]
    pub fn polish(&self) -> Option<&str> {
        self.polish.as_deref()
    }

    #[must_use]
    pub fn portuguese(&self) -> Option<&str> {
        self.portuguese.as_deref()
    }

    #[must_use]
    pub fn russian(&self) -> Option<&str> {
        self.russian.as_deref()
    }

    #[must_use]
    pub fn german(&self) -> Option<&str> {
        self.german.as_deref()
    }

    #[must_use]
    pub fn korean(&self) -> Option<&str> {
        self.korean.as_deref()
    }

    #[must_use]
    pub fn japanese(&self) -> Option<&str> {
        self.japanese.as_deref()
    }

    #[must_use]
    pub fn chinese(&self) -> Option<&str> {
        self.chinese.as_deref()
    }

    #[must_use]
    pub fn chinesesimp(&self) -> Option<&str> {
        self.chinesesimp.as_deref()
    }

    #[must_use]
    pub fn turkish(&self) -> Option<&str> {
        self.turkish.as_deref()
    }

    #[must_use]
    pub fn swedish(&self) -> Option<&str> {
        self.swedish.as_deref()
    }

    #[must_use]
    pub fn slovak(&self) -> Option<&str> {
        self.slovak.as_deref()
    }

    #[must_use]
    pub fn serbocroatian(&self) -> Option<&str> {
        self.serbocroatian.as_deref()
    }

    #[must_use]
    pub fn norwegian(&self) -> Option<&str> {
        self.norwegian.as_deref()
    }

    #[must_use]
    pub fn icelandic(&self) -> Option<&str> {
        self.icelandic.as_deref()
    }

    #[must_use]
    pub fn hungarian(&self) -> Option<&str> {
        self.hungarian.as_deref()
    }

    #[must_use]
    pub fn greek(&self) -> Option<&str> {
        self.greek.as_deref()
    }

    #[must_use]
    pub fn finnish(&self) -> Option<&str> {
        self.finnish.as_deref()
    }

    #[must_use]
    pub fn dutch(&self) -> Option<&str> {
        self.dutch.as_deref()
    }

    #[must_use]
    pub fn ukrainian(&self) -> Option<&str> {
        self.ukrainian.as_deref()
    }

    #[must_use]
    pub fn danish(&self) -> Option<&str> {
        self.danish.as_deref()
    }

    /// Set the value for a specific language.
    ///
    /// # Panics
    /// If the language is unknown or not supported.
    pub fn set(&mut self, language: &str, value: String) {
        match language.to_lowercase().as_str() {
            "original" | "english" => self.english = Some(value),
            "czech" => self.czech = Some(value),
            "french" => self.french = Some(value),
            "spanish" => self.spanish = Some(value),
            "italian" => self.italian = Some(value),
            "polish" => self.polish = Some(value),
            "portuguese" => self.portuguese = Some(value),
            "russian" => self.russian = Some(value),
            "german" => self.german = Some(value),
            "korean" => self.korean = Some(value),
            "japanese" => self.japanese = Some(value),
            "chinese" => self.chinese = Some(value),
            "chinesesimp" => self.chinesesimp = Some(value),
            "turkish" => self.turkish = Some(value),
            "swedish" => self.swedish = Some(value),
            "slovak" => self.slovak = Some(value),
            "serbocroatian" => self.serbocroatian = Some(value),
            "norwegian" => self.norwegian = Some(value),
            "icelandic" => self.icelandic = Some(value),
            "hungarian" => self.hungarian = Some(value),
            "greek" => self.greek = Some(value),
            "finnish" => self.finnish = Some(value),
            "dutch" => self.dutch = Some(value),
            "ukrainian" => self.ukrainian = Some(value),
            "danish" => self.danish = Some(value),
            _ => panic!("Unknown language: {language}"),
        }
    }
}
