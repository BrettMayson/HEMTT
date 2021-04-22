use serde::de::{DeserializeSeed, MapAccess};

use crate::error::Error;

pub struct ArmaClass<'a, 'de: 'a> {
    de: &'a mut crate::Deserializer<'de>,
}

impl<'a, 'de> ArmaClass<'a, 'de> {
    pub fn new(de: &'a mut crate::Deserializer<'de>) -> Self {
        ArmaClass { de }
    }
}

// `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// through entries of the map.
impl<'de, 'a> MapAccess<'de> for ArmaClass<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: DeserializeSeed<'de>,
    {
        // Check if there are no more entries.
        loop {
            let peek = self.de.peek_char();
            if crate::WHITESPACE.contains(peek) {
                self.de.next_char();
            } else {
                break;
            }
        }
        if self.de.peek_char() == '}' {
            self.de.next_char();
            return Ok(None);
        }

        if self.de.peek_char() == '?' {
            return Ok(None);
        }

        if self.de.input.starts_with("class ") {
            self.de.input = &self.de.input["class ".len()..];
            self.de.next_is_class = true;
            loop {
                if crate::WHITESPACE.contains(self.de.peek_char()) {
                    self.de.next_char();
                } else {
                    break;
                }
            }
        }

        // Deserialize a map key.
        self.de.next_is_key = true;
        let key = seed.deserialize(&mut *self.de).map(Some);
        loop {
            if crate::WHITESPACE.contains(self.de.peek_char()) {
                self.de.next_char();
            } else {
                break;
            }
        }
        self.de.next_is_key = false;
        key
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: DeserializeSeed<'de>,
    {
        // clear white space
        loop {
            if crate::WHITESPACE.contains(self.de.peek_char()) {
                self.de.next_char();
            } else {
                break;
            }
        }
        if !self.de.next_is_class && self.de.next_char() != '=' {
            return Err(Error::ExpectedEquals);
        }
        loop {
            if crate::WHITESPACE.contains(self.de.peek_char()) {
                self.de.next_char();
            } else {
                break;
            }
        }
        // Deserialize a map value.
        let value = seed.deserialize(&mut *self.de);
        loop {
            if crate::WHITESPACE.contains(self.de.peek_char()) {
                self.de.next_char();
            } else {
                break;
            }
        }

        if self.de.next_char() != ';' {
            return Err(Error::ExpectedSemiColon);
        }

        value
    }
}
