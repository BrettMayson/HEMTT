use serde::Deserialize;

use std::collections::HashMap;

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct InternalArmaMission {
    version: u8,
    binarizationWanted: u8,
    addons: Vec<String>,
    AddonsMetaData: AddonsMetaData,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct AddonsMetaData {
    List: AddonsMetaDataList,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct AddonsMetaDataList {
    #[serde(rename="items")]
    item_count: u8,
    #[serde(flatten)]
    items: HashMap<String, AddonsMetaDataListItem>,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct AddonsMetaDataListItem {
    className: String,
    name: String,
    author: Option<String>,
    url: Option<String>,
}
