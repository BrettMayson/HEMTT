use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use pbr::ProgressBar;
use serde_xml_rs;
use serde::Deserialize;
use serde_xml_rs;
use strum::IntoEnumIterator;
use walkdir::WalkDir;

#[cfg(not(windows))]
use indicatif::ProgressBar;
#[cfg(windows)]
use indicatif_windows::ProgressBar;

use crate::{AddonLocation, Command, HEMTTError};

pub struct Translation {}
impl Command for Translation {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("translation").about("Get translation info from `stringtable.xml` files")
    }

    fn require_project(&self) -> bool {
        false
    }

    fn run_no_project(&self, _args: &clap::ArgMatches) -> Result<(), HEMTTError> {
        let mut stringtables = Vec::new();
        for location in AddonLocation::iter() {
            stringtables.append(&mut Translation::get_stringtables(&location.to_path_buf()));
        }
        let (total, keys) = Translation::analyze(stringtables)?;
        println!("{:<15} {:>5}", "Total", total);
        let mut count_vec: Vec<(&String, &f64)> = keys.iter().collect();
        count_vec.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
        for (lang, count) in count_vec {
            println!("{:<15} {:>5} {:>3.0}%", lang, count, count / total * 100.0);
        }
        Ok(())
    }
}

impl Translation {
    /// Walk a folder to get `stringtable.xml` files
    pub fn get_stringtables(path: &PathBuf) -> Vec<PathBuf> {
        let mut stringtables = Vec::new();
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path().to_path_buf();
            if !path.ends_with("stringtable.xml") {
                continue;
            };
            stringtables.push(path.to_owned());
        }
        stringtables
    }

    pub fn analyze(stringtables: Vec<PathBuf>) -> Result<(f64, HashMap<String, f64>), HEMTTError> {
        let mut total = 0.0;
        let mut keys = HashMap::new();
        let pb = ProgressBar::new(stringtables.len() as u64);
        for stringtable in stringtables {
            let f = BufReader::new(open_file!(stringtable)?);
            let project: Project = serde_xml_rs::from_reader(f)
                .map_err(|_| HEMTTError::SIMPLE(format!("Unable to parse stringtable `{:?}`", stringtable)))?;
            for mut package in project.packages {
                package = package.transfer();
                for container in package.containers {
                    for key in container.keys {
                        total += 1.0;
                        for lang in key.languages {
                            match lang {
                                Language::Original(_) => *keys.entry("Original".to_owned()).or_insert(0.0) += 1.0,
                                Language::English(_) => *keys.entry("English".to_owned()).or_insert(0.0) += 1.0,
                                Language::Czech(_) => *keys.entry("Czech".to_owned()).or_insert(0.0) += 1.0,
                                Language::French(_) => *keys.entry("French".to_owned()).or_insert(0.0) += 1.0,
                                Language::German(_) => *keys.entry("German".to_owned()).or_insert(0.0) += 1.0,
                                Language::Italian(_) => *keys.entry("Italian".to_owned()).or_insert(0.0) += 1.0,
                                Language::Polish(_) => *keys.entry("Polish".to_owned()).or_insert(0.0) += 1.0,
                                Language::Portuguese(_) => *keys.entry("Portuguese".to_owned()).or_insert(0.0) += 1.0,
                                Language::Russian(_) => *keys.entry("Russian".to_owned()).or_insert(0.0) += 1.0,
                                Language::Spanish(_) => *keys.entry("Spanish".to_owned()).or_insert(0.0) += 1.0,
                                Language::Korean(_) => *keys.entry("Korean".to_owned()).or_insert(0.0) += 1.0,
                                Language::Japanese(_) => *keys.entry("Japanese".to_owned()).or_insert(0.0) += 1.0,
                                Language::Chinesesimp(_) => *keys.entry("Chinesesimp".to_owned()).or_insert(0.0) += 1.0,
                                Language::Chinese(_) => *keys.entry("Chinese".to_owned()).or_insert(0.0) += 1.0,
                                Language::Turkish(_) => *keys.entry("Turkish".to_owned()).or_insert(0.0) += 1.0,
                                Language::Hungarian(_) => *keys.entry("Hungarian".to_owned()).or_insert(0.0) += 1.0,
                                Language::Swedish(_) => *keys.entry("Swedish".to_owned()).or_insert(0.0) += 1.0,
                                Language::Slovak(_) => *keys.entry("Slovak".to_owned()).or_insert(0.0) += 1.0,
                                Language::SerboCroatian(_) => *keys.entry("SerboCroatian".to_owned()).or_insert(0.0) += 1.0,
                                Language::Norwegian(_) => *keys.entry("Norwegian".to_owned()).or_insert(0.0) += 1.0,
                                Language::Icelandic(_) => *keys.entry("Icelandic".to_owned()).or_insert(0.0) += 1.0,
                                Language::Greek(_) => *keys.entry("Greek".to_owned()).or_insert(0.0) += 1.0,
                                Language::Finnish(_) => *keys.entry("Finnish".to_owned()).or_insert(0.0) += 1.0,
                                Language::Dutch(_) => *keys.entry("Dutch".to_owned()).or_insert(0.0) += 1.0,
                            }
                        }
                    }
                }
            }
            pb.inc(1);
        }
        pb.finish_and_clear();
        Ok((total, keys))
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone, Hash, Eq)]
enum Language {
    Original(String),
    English(String),
    Czech(String),
    French(String),
    German(String),
    Italian(String),
    Polish(String),
    Portuguese(String),
    Russian(String),
    Spanish(String),
    Korean(String),
    Japanese(String),
    Chinesesimp(String),
    Chinese(String),
    Turkish(String),
    Hungarian(String),
    Swedish(String),
    Slovak(String),
    SerboCroatian(String),
    Norwegian(String),
    Icelandic(String),
    Greek(String),
    Finnish(String),
    Dutch(String),
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
struct Key {
    #[serde(rename = "$value")]
    pub languages: Vec<Language>,
}

#[derive(Debug, Deserialize)]
struct Container {
    #[serde(rename = "Key")]
    pub keys: Vec<Key>,
}

#[derive(Debug, Deserialize)]
struct Package {
    #[serde(rename = "Container")]
    #[serde(default = "Vec::new")]
    pub containers: Vec<Container>,

    #[serde(rename = "Key")]
    #[serde(default = "Vec::new")]
    pub keys: Vec<Key>,
}
impl Package {
    fn transfer(mut self) -> Package {
        if !self.keys.is_empty() {
            let keys = &self.keys;
            self.containers.push(Container { keys: keys.to_vec() });
        }
        self
    }
}

#[derive(Debug, Deserialize)]
struct Project {
    #[serde(rename = "Package")]
    pub packages: Vec<Package>,
}
