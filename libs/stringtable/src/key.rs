use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Key {
    #[serde(rename = "@ID")]
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    original: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    english: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    czech: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    french: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    spanish: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    italian: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    polish: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    portuguese: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    russian: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    german: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    korean: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    japanese: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chinese: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chinesesimp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    turkish: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    swedish: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    slovak: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    serbocroatian: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    norwegian: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icelandic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hungarian: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    greek: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    finnish: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dutch: Option<String>,
}

impl Key {
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
}
