use serde::de::{DeserializeSeed, SeqAccess};

use crate::error::Error;

pub struct CommaSeparated<'a, 'de: 'a> {
    de: &'a mut crate::Deserializer<'de>,
    first: bool,
}

impl<'a, 'de> CommaSeparated<'a, 'de> {
    pub fn new(de: &'a mut crate::Deserializer<'de>) -> Self {
        CommaSeparated { de, first: true }
    }
}

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'de, 'a> SeqAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        loop {
            if crate::WHITESPACE.contains(self.de.peek_char()) {
                self.de.next_char();
            } else {
                break;
            }
        }
        // Check if there are no more elements.
        if self.de.peek_char() == '}' {
            return Ok(None);
        }
        loop {
            if crate::WHITESPACE.contains(self.de.peek_char()) {
                self.de.next_char();
            } else {
                break;
            }
        }
        // Comma is required before every element except the first.
        if !self.first && self.de.next_char() != ',' {
            return Err(Error::ExpectedArrayComma);
        }
        self.first = false;
        loop {
            if crate::WHITESPACE.contains(self.de.peek_char()) {
                self.de.next_char();
            } else {
                break;
            }
        }
        // Deserialize an array element.
        seed.deserialize(&mut *self.de).map(Some)
    }
}
