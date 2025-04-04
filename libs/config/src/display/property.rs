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
                    let equals = if matches!(value, crate::Value::Array(Array { expand: true, .. }))
                    {
                        "+="
                    } else {
                        "="
                    };
                    writeln!(f, "{name}[] {equals} {value};")
                } else {
                    writeln!(f, "{name} = {value};")
                }
            }
            Self::Delete(name) => {
                writeln!(f, "delete {name};")
            }
            Self::Class(class) => {
                write!(f, "{class}")
            }
            Self::MissingSemicolon(_, _) => {
                unreachable!()
            }
        }
    }
}
