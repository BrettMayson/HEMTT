use std::fmt::{self, Display};

use serde::{de, ser};

// This is a bare-bones implementation. A real library would provide additional
// information in its error type, for example the line and column at which the
// error occurred, the byte offset into the input, or the current key being
// processed.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    // One or more variants that can be created by data structures through the
    // `ser::Error` and `de::Error` traits. For example the Serialize impl for
    // Mutex<T> might return an error because the mutex is poisoned, or the
    // Deserialize impl for a struct may return an error because a required
    // field is missing.
    Message(String),

    Eof,
    Syntax,
    ExpectedSemiColon,
    ExpectedEquals,
    ExpectedArrayComma,
    ExpectedString,
    TrailingCharacters,
    // may remove
    ExpectedMapComma,
    ExpectedMapColon,
    ExpectedMapEnd,
    ExpectedMap,
    ExpectedEnum,
    ExpectedArray,
    ExpectedArrayEnd,
    ExpectedNull,
    ExpectedBoolean,
    ExpectedInteger,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(&<dyn std::error::Error>::to_string(self))
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Message(ref msg) => msg,
            Error::Eof => "unexpected end of input",
            _ => "no error messages eh",
        }
    }
}
