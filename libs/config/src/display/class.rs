use crate::Class;

impl std::fmt::Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Root { properties } => {
                for property in properties {
                    write!(f, "{property}")?;
                }
                Ok(())
            }
            Self::Local {
                name,
                parent,
                properties,
                ..
            } => {
                if let Some(parent) = parent {
                    write!(f, "class {name}: {parent} {{")?;
                } else {
                    write!(f, "class {name} {{")?;
                }
                if !self.properties().is_empty() {
                    writeln!(f)?;
                }
                let mut properties_str = String::new();
                for property in properties {
                    properties_str.push_str(&property.to_string());
                }
                for line in properties_str.lines() {
                    writeln!(f, "    {line}")?;
                }
                writeln!(f, "}};")?;
                Ok(())
            }
            Self::External { name } => {
                writeln!(f, "class {name};")
            }
        }
    }
}
