use chumsky::{prelude::*, Parser};

pub enum State {
    Properties,
    ClassName,
    ClassExternal,
}

pub fn format(content: &str) -> Result<String, String> {
    let config = config().parse(content).map_err(|e| {
        e.into_iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    Ok(config.write())
}

pub fn config() -> impl Parser<char, Config, Error = Simple<char>> {
    choice((
        property()
            .padded()
            .repeated()
            .delimited_by(empty(), end())
            .map(Config),
        end().padded().map(|()| Config(vec![])),
    ))
}

fn property() -> impl Parser<char, Property, Error = Simple<char>> {
    recursive(|rec| {
        let class = just("class ")
            .padded()
            .ignore_then(ident().padded().labelled("class name"))
            .then(
                (just(':')
                    .padded()
                    .ignore_then(ident().padded().labelled("class parent")))
                .or_not(),
            )
            .padded()
            .then(
                rec.labelled("class property")
                    .padded()
                    .repeated()
                    .padded()
                    .delimited_by(just('{'), just('}'))
                    .or_not(),
            )
            .map(|((ident, parent), properties)| {
                if let Some(properties) = properties {
                    Class {
                        name: ident,
                        parent,
                        external: false,
                        properties,
                    }
                } else {
                    Class {
                        name: ident,
                        parent,
                        external: true,
                        properties: vec![],
                    }
                }
            });
        choice((
            choice((
                class.map(Property::Class),
                choice((
                    ident()
                        .padded()
                        .then_ignore(just('=').padded())
                        .then(
                            value(";}\n".to_string())
                                .or(none_of(";}")
                                    .repeated()
                                    .at_least(1)
                                    .collect::<String>()
                                    .map(|s| s))
                                .map(|s| vec![s]),
                        )
                        .map(|(name, values)| Value { name, values }),
                    // an array of values
                    ident()
                        .padded()
                        .then_ignore(just("[]").padded())
                        .then_ignore(just('=').padded())
                        .then(
                            value(";}".to_string())
                                .padded()
                                .separated_by(just(',').padded())
                                .allow_trailing()
                                .delimited_by(just('{'), just('}')),
                        )
                        .map(|(name, values)| Value::new(name, values)),
                ))
                .map(Property::Value),
            ))
            .then_ignore(one_of(";\n").padded().or_not()),
            just("#").padded().ignore_then(
                none_of("\n")
                    .repeated()
                    .at_least(1)
                    .collect::<String>()
                    .map(|s| Property::Directive(format!("#{s}"))),
            ),
        ))
    })
}

fn ident() -> impl Parser<char, String, Error = Simple<char>> {
    one_of("0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_()")
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(|s| s)
}

fn value(end: String) -> impl Parser<char, String, Error = Simple<char>> {
    choice((
        string('"'),
        none_of(end)
            .repeated()
            .at_least(1)
            .collect::<String>()
            .map(|s| s),
    ))
    .padded()
}

fn string(delimiter: char) -> impl Parser<char, String, Error = Simple<char>> {
    let content = just(delimiter).not().or(just([delimiter; 2]).to(delimiter));
    let segment = just(delimiter)
        .ignore_then(content.repeated())
        .then_ignore(just(delimiter))
        .collect();
    segment
        .separated_by(just("\\n").padded())
        .at_least(1)
        .collect::<Vec<String>>()
        .map(|s| {
            format!(
                "\"{}\"",
                s.into_iter()
                    .collect::<Vec<_>>()
                    .join("\n")
                    .replace("\\\n", "")
            )
        })
}

#[derive(Debug, PartialEq)]
pub struct Config(Vec<Property>);

impl Config {
    #[must_use]
    pub fn write(&self) -> String {
        self.0
            .iter()
            .map(|p| p.write(0))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug, PartialEq)]
pub enum Property {
    Class(Class),
    Value(Value),
    Directive(String),
}

impl Property {
    #[must_use]
    pub fn write(&self, indent: u8) -> String {
        match self {
            Self::Class(c) => c.write(indent),
            Self::Value(v) => v.write(indent),
            Self::Directive(d) => d.clone(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Class {
    pub name: String,
    pub external: bool,
    pub parent: Option<String>,
    pub properties: Vec<Property>,
}

impl Class {
    pub fn write(&self, indent: u8) -> String {
        let indent_str = " ".repeat(4 * indent as usize);
        if self.external {
            self.parent.as_ref().map_or_else(
                || format!("{}class {};", indent_str, self.name),
                |parent| format!("{}class {}: {} {{}};", indent_str, self.name, parent),
            )
        } else {
            let parent = self
                .parent
                .as_ref()
                .map_or_else(String::new, |parent| format!(": {parent}"));
            format!(
                "{}class {}{} {{\n{}\n{}}};",
                indent_str,
                self.name,
                parent,
                self.properties
                    .iter()
                    .map(|p| p.write(indent + 1))
                    .collect::<Vec<_>>()
                    .join("\n"),
                indent_str
            )
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
/// Either a single value or an array of values
///
/// ```text
/// name = "Test"
/// names[] = { "Bob", "Alice" }
/// ```
pub struct Value {
    pub name: String,
    pub values: Vec<String>,
}

impl Value {
    #[must_use]
    pub fn new(name: String, values: Vec<String>) -> Self {
        // see if the values are a string that contain multiple values
        // "1, 2, 3" -> ["1", "2", "3"]
        // respect parenthesis and strings
        // "myFunc(1, 2), 3" -> ["myFunc(1, 2)", "3"]
        let values = values
            .into_iter()
            .flat_map(|v| {
                if v.contains(',') {
                    let mut values = vec![];
                    let mut current = String::new();
                    let mut in_string = false;
                    let mut in_parenthesis = 0;
                    for c in v.chars() {
                        match c {
                            '"' => in_string = !in_string,
                            '(' => in_parenthesis += 1,
                            ')' => in_parenthesis -= 1,
                            ',' if !in_string && in_parenthesis == 0 => {
                                values.push(current.trim().to_string());
                                current.clear();
                            }
                            _ => current.push(c),
                        }
                    }
                    values.push(current.trim().to_string());
                    values
                } else {
                    vec![v]
                }
            })
            .collect();
        Self { name, values }
    }

    #[must_use]
    pub fn write(&self, indent: u8) -> String {
        let indent = " ".repeat(4 * indent as usize);
        if self.values.len() == 1 {
            format!("{}{} = {};", indent, self.name, self.values[0])
        } else {
            format!(
                "{}{}[] = {{{}}};",
                indent,
                self.name,
                // use a single line if less than 60 characters
                if self
                    .values
                    .iter()
                    .map(std::string::String::len)
                    .sum::<usize>()
                    + self.values.len()
                    < 60
                {
                    self.values.join(", ")
                } else {
                    format!(
                        "\n{}\n{indent}",
                        self.values
                            .iter()
                            .map(|v| format!("{indent}    {v}"))
                            .collect::<Vec<_>>()
                            .join(",\n")
                    )
                }
            )
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_string() {
        let input = "name = \"Test\"";
        let result = property().parse(input).unwrap();
        assert_eq!(
            result,
            Property::Value(Value {
                name: "name".to_string(),
                values: vec!["Test".to_string()]
            })
        );
    }

    #[test]
    fn property_string_array() {
        let input = "names[] = { \"Bob\", \"Alice\" }";
        let result = property().parse(input).unwrap();
        assert_eq!(
            result,
            Property::Value(Value {
                name: "names".to_string(),
                values: vec!["Bob".to_string(), "Alice".to_string()]
            })
        );
    }

    #[test]
    fn property_number_array() {
        let input = "numbers[] = { 1, 2, 3 }";
        let result = property().parse(input).unwrap();
        assert_eq!(
            result,
            Property::Value(Value {
                name: "numbers".to_string(),
                values: vec!["1".to_string(), "2".to_string(), "3".to_string()]
            })
        );
    }

    #[test]
    fn property_garbage() {
        let input = "name = just a bunch of garbage";
        let result = property().parse(input).unwrap();
        assert_eq!(
            result,
            Property::Value(Value {
                name: "name".to_string(),
                values: vec!["just a bunch of garbage".to_string()]
            })
        );
    }

    #[test]
    fn property_class() {
        let input = "class Test: Parent { name = \"Test\"; }";
        let result = property().parse(input).unwrap();
        assert_eq!(
            result,
            Property::Class(Class {
                name: "Test".to_string(),
                external: false,
                parent: Some("Parent".to_string()),
                properties: vec![Property::Value(Value {
                    name: "name".to_string(),
                    values: vec!["Test".to_string()]
                })]
            })
        );
    }

    #[test]
    fn multiple_properties() {
        let input = r#"name = "Test"; names[] = { "Bob", "Alice" }"#;
        let result = config().parse(input).unwrap();
        assert_eq!(
            result,
            Config(vec![
                Property::Value(Value {
                    name: "name".to_string(),
                    values: vec!["Test".to_string()]
                }),
                Property::Value(Value {
                    name: "names".to_string(),
                    values: vec!["Bob".to_string(), "Alice".to_string()]
                })
            ])
        );
    }

    #[test]
    fn multiple_classes() {
        let input = r#"class Test: Parent { name = "Test"; }; class Test2 { name = "Test2"; };"#;
        let result = config().parse(input).unwrap();
        assert_eq!(
            result,
            Config(vec![
                Property::Class(Class {
                    name: "Test".to_string(),
                    external: false,
                    parent: Some("Parent".to_string()),
                    properties: vec![Property::Value(Value {
                        name: "name".to_string(),
                        values: vec!["Test".to_string()]
                    })]
                }),
                Property::Class(Class {
                    name: "Test2".to_string(),
                    external: false,
                    parent: None,
                    properties: vec![Property::Value(Value {
                        name: "name".to_string(),
                        values: vec!["Test2".to_string()]
                    })]
                })
            ])
        );
    }

    #[test]
    fn nested_external() {
        let input = r"class Something { class External; };";
        let result = config().parse(input).unwrap();
        assert_eq!(
            result,
            Config(vec![Property::Class(Class {
                name: "Something".to_string(),
                external: false,
                parent: None,
                properties: vec![Property::Class(Class {
                    name: "External".to_string(),
                    external: true,
                    parent: None,
                    properties: vec![]
                })]
            })])
        );
    }

    #[test]
    fn ace_throwing_ammo() {
        let input = r#"class CfgAmmo {
    class Default;
    class Grenade: Default {
        GVAR(torqueDirection)[] = {1, 1, 0};
        GVAR(torqueMagnitude) = "(50 + random 100) * selectRandom [1, -1]";
    };
    class GrenadeCore: Default {
        GVAR(torqueDirection)[] = {1, 1, 0};
        GVAR(torqueMagnitude) = "(50 + random 100) * selectRandom [1, -1]";
    };
};
"#;
        let result = config().parse(input).unwrap();
        assert_eq!(
            result,
            Config(vec![Property::Class(Class {
                name: "CfgAmmo".to_string(),
                external: false,
                parent: None,
                properties: vec![
                    Property::Class(Class {
                        name: "Default".to_string(),
                        external: false,
                        parent: None,
                        properties: vec![]
                    }),
                    Property::Class(Class {
                        name: "Grenade".to_string(),
                        external: false,
                        parent: Some("Default".to_string()),
                        properties: vec![
                            Property::Value(Value {
                                name: "GVAR(torqueDirection)".to_string(),
                                values: vec!["1".to_string(), "1".to_string(), "0".to_string()]
                            }),
                            Property::Value(Value {
                                name: "GVAR(torqueMagnitude)".to_string(),
                                values: vec!["(50 + random 100) * selectRandom [1, -1]".to_string()]
                            })
                        ]
                    }),
                    Property::Class(Class {
                        name: "GrenadeCore".to_string(),
                        external: false,
                        parent: Some("Default".to_string()),
                        properties: vec![
                            Property::Value(Value {
                                name: "GVAR(torqueDirection)".to_string(),
                                values: vec!["1".to_string(), "1".to_string(), "0".to_string()]
                            }),
                            Property::Value(Value {
                                name: "GVAR(torqueMagnitude)".to_string(),
                                values: vec!["(50 + random 100) * selectRandom [1, -1]".to_string()]
                            })
                        ]
                    })
                ]
            })])
        );
    }
}
