use crate::{Array, Item};

impl std::fmt::Display for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        if self.items.is_empty() {
            return write!(f, "}}");
        }
        let mut items = Vec::new();
        for item in self.items.iter() {
            items.push(item.to_string());
        }
        if items.len() <= 3 {
            write!(f, "{}", items.join(", "))?;
        } else {
            let str = items.join(", ");
            if str.len() > 50 {
                writeln!(f)?;
                write!(f, "    {}", items.join(",\n    "))?;
                writeln!(f)?;
            } else {
                write!(f, "{str}")?;
            }
        }
        write!(f, "}}")
    }
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Str(str) => write!(f, "{str}"),
            Self::Number(number) => write!(f, "{number}"),
            Self::Array(items) => {
                write!(f, "{{")?;
                let mut first = true;
                for item in items {
                    if !first {
                        write!(f, ", ")?;
                    }
                    first = false;
                    write!(f, "{}", item.inner)?;
                }
                write!(f, "}}")
            }
            Self::Invalid(_) => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chumsky::span::{SimpleSpan, Spanned};

    use super::*;
    use crate::{Number, Str};

    #[test]
    fn test_array_one_item() {
        let array = Array::test_new(vec![Item::Str(Str(Arc::from("value")))]);
        assert_eq!(array.to_string(), "{\"value\"}");
    }

    #[test]
    fn test_array_multiple_items() {
        let array = Array::test_new(vec![
            Item::Str(Str(Arc::from("value1"))),
            Item::Str(Str(Arc::from("value2"))),
            Item::Str(Str(Arc::from("value3"))),
        ]);
        assert_eq!(array.to_string(), "{\"value1\", \"value2\", \"value3\"}");
    }

    #[test]
    fn test_array_empty() {
        let array = Array::test_new(vec![]);
        assert_eq!(array.to_string(), "{}");
    }

    #[test]
    fn test_array_nested() {
        let array = Array::test_new(vec![
            Item::Str(Str(Arc::from("value1"))),
            Item::Array(vec![Spanned {
                inner: Item::Str(Str(Arc::from("nested"))),
                span: SimpleSpan::default(),
            }]),
        ]);
        assert_eq!(array.to_string(), "{\"value1\", {\"nested\"}}");
    }

    #[test]
    fn test_array_mix() {
        let array = Array::test_new(vec![
            Item::Str(Str(Arc::from("value1"))),
            Item::Number(Number::Float32(42f32)),
            Item::Array(vec![Spanned {
                inner: Item::Str(Str(Arc::from("nested"))),
                span: SimpleSpan::default(),
            }]),
        ]);
        assert_eq!(array.to_string(), "{\"value1\", 42, {\"nested\"}}");
    }
}
