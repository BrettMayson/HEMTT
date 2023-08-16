use hemtt_common::workspace::WorkspacePath;

use crate::{processor::Processor, token::Token, Error};

#[derive(Debug, Default)]
pub struct Processed {
    pub(crate) tokens: Vec<Token>,
}

impl Processed {
    pub fn new(path: &WorkspacePath) -> Result<Self, Error> {
        Processor::run(path)
    }

    pub fn to_source(&self) -> String {
        self.tokens
            .iter()
            .map(|t| t.to_source())
            .collect::<Vec<_>>()
            .join("")
    }
}

impl ToString for Processed {
    fn to_string(&self) -> String {
        self.tokens
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join("")
    }
}
