use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use pbr::ProgressBar;
use serde_xml_rs;
use serde::Deserialize;
use walkdir::WalkDir;

use crate::error::*;
use crate::project::use_project_dir;

pub fn check() -> Result<(), std::io::Error> {
    use_project_dir();
    let mut total = 0.0;
    let mut keys = HashMap::new();
    let mut stringtables = Vec::new();
    for entry in WalkDir::new("addons").into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();
        if !path.ends_with("stringtable.xml") { continue };
        stringtables.push(path);
    }
    for entry in WalkDir::new("optionals").into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();
        if !path.ends_with("stringtable.xml") { continue };
        stringtables.push(path);
    }
    let mut pb = ProgressBar::new(stringtables.len() as u64);
    pb.show_speed = false;
    for stringtable in stringtables {
        pb.inc();
        let f = BufReader::new(open_file!(stringtable)?);
        let project: Project = serde_xml_rs::from_reader(f).unwrap_or_print();
        for mut package in project.packages {
            package = package.transfer();
            for container in package.containers {
                for key in container.keys {
                    total += 1.0;
                    for lang in key.languages {
                        match lang {
                            Language::Original(_) => *keys.entry("Original").or_insert(0.0) += 1.0,
                            Language::English(_) => *keys.entry("English").or_insert(0.0) += 1.0,
                            Language::Czech(_) => *keys.entry("Czech").or_insert(0.0) += 1.0,
                            Language::French(_) => *keys.entry("French").or_insert(0.0) += 1.0,
                            Language::German(_) => *keys.entry("German").or_insert(0.0) += 1.0,
                            Language::Italian(_) => *keys.entry("Italian").or_insert(0.0) += 1.0,
                            Language::Polish(_) => *keys.entry("Polish").or_insert(0.0) += 1.0,
                            Language::Portuguese(_) => *keys.entry("Portuguese").or_insert(0.0) += 1.0,
                            Language::Russian(_) => *keys.entry("Russian").or_insert(0.0) += 1.0,
                            Language::Spanish(_) => *keys.entry("Spanish").or_insert(0.0) += 1.0,
                            Language::Korean(_) => *keys.entry("Korean").or_insert(0.0) += 1.0,
                            Language::Japanese(_) => *keys.entry("Japanese").or_insert(0.0) += 1.0,
                            Language::Chinesesimp(_) => *keys.entry("Chinesesimp").or_insert(0.0) += 1.0,
                            Language::Chinese(_) => *keys.entry("Chinese").or_insert(0.0) += 1.0,
                            Language::Turkish(_) => *keys.entry("Turkish").or_insert(0.0) += 1.0,
                            Language::Hungarian(_) => *keys.entry("Hungarian").or_insert(0.0) += 1.0,
                            Language::Swedish(_) => *keys.entry("Swedish").or_insert(0.0) += 1.0,
                            Language::Slovak(_) => *keys.entry("Slovak").or_insert(0.0) += 1.0,
                            Language::SerboCroatian(_) => *keys.entry("SerboCroatian").or_insert(0.0) += 1.0,
                            Language::Norwegian(_) => *keys.entry("Norwegian").or_insert(0.0) += 1.0,
                            Language::Icelandic(_) => *keys.entry("Icelandic").or_insert(0.0) += 1.0,
                            Language::Greek(_) => *keys.entry("Greek").or_insert(0.0) += 1.0,
                            Language::Finnish(_) => *keys.entry("Finnish").or_insert(0.0) += 1.0,
                            Language::Dutch(_) => *keys.entry("Dutch").or_insert(0.0) += 1.0,
                        }
                    }
                }
            }
        }
    }
    pb.finish_print(&format!("{:<15} {:>5}", "Total", total));
    println!();
    let mut count_vec: Vec<(&&str, &f64)> = keys.iter().collect();
    count_vec.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    for (lang, count) in count_vec {
        println!("{:<15} {:>5} {:>3.0}%", lang, count, count / total * 100.0);
    }
    Ok(())
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
    #[serde(rename="Key")]
    pub keys: Vec<Key>
}

#[derive(Debug, Deserialize)]
struct Package {
    #[serde(rename="Container")]
    #[serde(default="Vec::new")]
    pub containers: Vec<Container>,

    #[serde(rename="Key")]
    #[serde(default="Vec::new")]
    pub keys: Vec<Key>
}
impl Package {
    fn transfer(mut self) -> Package {
        if !self.keys.is_empty() {
            let keys = &self.keys;
            self.containers.push(Container {
                keys: keys.to_vec(),
            });
        }
        self
    }
}

#[derive(Debug, Deserialize)]
struct Project {
    #[serde(rename="Package")]
    pub packages: Vec<Package>
}
