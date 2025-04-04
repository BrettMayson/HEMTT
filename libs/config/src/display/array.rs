use crate::{Array, Item};

impl std::fmt::Display for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        if self.items.is_empty() {
            return write!(f, "}}");
        }
        let mut items = Vec::new();
        for item in &self.items {
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
                    write!(f, "{item}")?;
                }
                write!(f, "}}")
            }
            Self::Invalid(_) => unreachable!(),
        }
    }
}
