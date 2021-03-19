use std::collections::HashMap;

use crate::{MipMap, PaXType};

#[derive(Debug)]
pub struct PAA {
    pub format: PaXType,
    pub taggs: HashMap<String, Vec<u8>>,
    pub maps: Vec<MipMap>,
}

impl PAA {
    pub fn new(format: PaXType) -> Self {
        Self {
            format,
            taggs: HashMap::new(),
            maps: Vec::new(),
        }
    }
}
