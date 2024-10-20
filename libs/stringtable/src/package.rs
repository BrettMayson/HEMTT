use serde::{Deserialize, Serialize};

use crate::{Key, Totals};

#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "Key")]
    #[serde(default)]
    keys: Vec<Key>,
    #[serde(rename = "Container")]
    #[serde(default)]
    containers: Vec<Package>,
}

impl Package {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn keys(&self) -> &[Key] {
        &self.keys
    }

    #[must_use]
    pub fn containers(&self) -> &[Self] {
        &self.containers
    }

    pub fn sort(&mut self) {
        self.keys.sort_by(|a, b| a.id().cmp(b.id()));
        for container in &mut self.containers {
            container.sort();
        }
    }

    #[must_use]
    pub fn totals(&self) -> Totals {
        let mut totals = Totals::default();
        for key in &self.keys {
            totals.inc();
            if key.original().is_some() {
                totals.inc_original();
            }
            if key.english().is_some() {
                totals.inc_english();
            }
            if key.czech().is_some() {
                totals.inc_czech();
            }
            if key.french().is_some() {
                totals.inc_french();
            }
            if key.spanish().is_some() {
                totals.inc_spanish();
            }
            if key.italian().is_some() {
                totals.inc_italian();
            }
            if key.polish().is_some() {
                totals.inc_polish();
            }
            if key.portuguese().is_some() {
                totals.inc_portuguese();
            }
            if key.russian().is_some() {
                totals.inc_russian();
            }
            if key.german().is_some() {
                totals.inc_german();
            }
            if key.korean().is_some() {
                totals.inc_korean();
            }
            if key.japanese().is_some() {
                totals.inc_japanese();
            }
            if key.chinese().is_some() {
                totals.inc_chinese();
            }
            if key.chinesesimp().is_some() {
                totals.inc_chinesesimp();
            }
            if key.turkish().is_some() {
                totals.inc_turkish();
            }
            if key.swedish().is_some() {
                totals.inc_swedish();
            }
            if key.slovak().is_some() {
                totals.inc_slovak();
            }
            if key.serbocroatian().is_some() {
                totals.inc_serbocroatian();
            }
            if key.norwegian().is_some() {
                totals.inc_norwegian();
            }
            if key.icelandic().is_some() {
                totals.inc_icelandic();
            }
            if key.hungarian().is_some() {
                totals.inc_hungarian();
            }
            if key.greek().is_some() {
                totals.inc_greek();
            }
            if key.finnish().is_some() {
                totals.inc_finnish();
            }
            if key.dutch().is_some() {
                totals.inc_dutch();
            }
        }
        for container in &self.containers {
            totals.merge(&container.totals());
        }
        totals
    }
}
