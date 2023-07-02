use crate::{Class, Property, Value};

use super::Rapify;

impl Rapify for Property {
    fn rapify<O: std::io::Write>(
        &self,
        _output: &mut O,
        _offset: usize,
    ) -> Result<usize, std::io::Error> {
        unreachable!()
    }

    fn rapified_length(&self) -> usize {
        self.property_code().len()
            + match self {
                Self::Entry { value, .. } => match value {
                    Value::Str(s) => s.rapified_length(),
                    Value::Number(n) => n.rapified_length(),
                    Value::Array(a) => a.rapified_length(),
                    Value::Invalid(_) => unreachable!(),
                },
                Self::Class(c) => match c {
                    Class::Local { .. } => 4,
                    Class::External { .. } => 0,
                },
                Self::Delete(_) => 0,
                Self::MissingSemicolon(_, _) => unreachable!(),
            }
    }
}

impl Property {
    /// Get the code of the property
    #[must_use]
    pub fn property_code(&self) -> Vec<u8> {
        match self {
            Self::Entry { value, .. } => match value {
                Value::Str(s) => vec![1, s.rapified_code()],
                Value::Number(n) => vec![1, n.rapified_code()],
                Value::Array(a) => {
                    if a.expand {
                        vec![5, 1, 0, 0, 0]
                    } else {
                        vec![2]
                    }
                }
                Value::Invalid(_) => unreachable!(),
            },
            Self::Class(c) => match c {
                Class::Local { .. } => vec![0],
                Class::External { .. } => vec![3],
            },
            Self::Delete(_) => {
                vec![4]
            }
            Self::MissingSemicolon(_, _) => unreachable!(),
        }
    }
}
