use chumsky::span::Spanned;

use crate::{Array, Property};

impl std::fmt::Display for Property {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Entry {
                name,
                value,
                expected_array,
            } => {
                if *expected_array {
                    let equals = if matches!(
                        value,
                        Spanned {
                            inner: crate::Value::Array(Array { expand: true, .. }),
                            ..
                        }
                    ) {
                        "+="
                    } else {
                        "="
                    };
                    writeln!(f, "{}[] {equals} {};", name.inner, value.inner)
                } else {
                    writeln!(f, "{} = {};", name.inner, value.inner)
                }
            }
            Self::Delete(name) => {
                writeln!(f, "delete {};", name.inner)
            }
            Self::Class(class) => {
                write!(f, "{}", class.inner)
            }
            Self::MissingSemicolon(_) | Self::ExtraSemicolons(_) => {
                unreachable!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chumsky::span::SimpleSpan;

    use super::*;
    use crate::{Ident, Str, Value};

    #[test]
    fn test_property_entry() {
        let property = Property::Entry {
            name: Spanned {
                inner: Ident(Arc::from("test")),
                span: SimpleSpan::default(),
            },
            value: Spanned {
                inner: Value::Str(Str(Arc::from("value"))),
                span: SimpleSpan::default(),
            },
            expected_array: false,
        };
        assert_eq!(property.to_string(), "test = \"value\";\n");
    }
}
