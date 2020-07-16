use std::collections::HashMap;
use std::path::PathBuf;

use semver::Version;
use serde::{Deserialize, Serialize};

use crate::defaults::*;

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct Project {
    pub name: String,
    pub prefix: String,
    pub author: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default = "String::new")]
    pub template: String,

    #[serde(default = "default_version")]
    pub version: Version,

    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default = "String::new")]
    pub modname: String,

    #[serde(default = "default_mainprefix")]
    pub mainprefix: String,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default = "HashMap::new")]
    #[serde(rename(deserialize = "headerexts"))] // DEPRECATED
    #[serde(rename(deserialize = "header_exts"))]
    pub header_exts: HashMap<String, String>,

    // Files
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "default_include")]
    pub include: Vec<PathBuf>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub exclude: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub files: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default = "default_folder_optionals")]
    pub folder_optionals: Option<bool>,

    // Signing
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default = "default_reuse_private_key")]
    pub reuse_private_key: Option<bool>,

    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default = "String::new")]
    #[serde(rename(deserialize = "keyname"))] // DEPRECATED
    #[serde(rename(deserialize = "key_name"))]
    key_name: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default = "String::new")]
    #[serde(rename(deserialize = "signame"))] // DEPRECATED
    #[serde(rename(deserialize = "sig_name"))] // DEPRECATED
    #[serde(rename(deserialize = "authority"))]
    pub authority: String,

    #[serde(default = "default_sig_version")]
    #[serde(rename(deserialize = "sigversion"))] // DEPRECATED
    #[serde(rename(deserialize = "sig_version"))]
    pub sig_version: u8,

    // Scripts
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub check: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub prebuild: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub postbuild: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub releasebuild: Vec<String>,
    // #[serde(skip_serializing_if = "HashMap::is_empty")]
    // #[serde(default = "HashMap::new")]
    // pub scripts: HashMap<String, crate::BuildScript>,
}
