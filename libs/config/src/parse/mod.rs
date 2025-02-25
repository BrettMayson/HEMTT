//! # Parse

use chumsky::prelude::*;

use crate::Config;

use self::property::property;

mod array;
mod ident;
mod number;
mod property;
mod str;
mod value;

/// Parse a config file.
pub fn config<'a>() -> impl Parser<'a, &'a str, Config, extra::Err<Rich<'a, char>>> {
    choice((
        property()
            .padded()
            .repeated()
            .collect::<Vec<_>>()
            .delimited_by(empty(), end())
            .map(Config),
        end().padded().map(|()| Config(vec![])),
    ))
}

#[cfg(test)]
mod tests {
    use chumsky::Parser;

    use crate::{parse::config, Config};

    #[test]
    fn empty() {
        assert_eq!(config().parse(r"",).output(), Some(&Config(vec![]),));
        assert_eq!(config().parse(r"   ",).output(), Some(&Config(vec![]),));
    }

    #[test]
    fn single_item() {
        assert_eq!(
            config().parse(r#"MyData = "Hello World";"#,).output(),
            Some(&Config(vec![crate::Property::Entry {
                name: crate::Ident {
                    value: "MyData".to_string(),
                    span: 0..6,
                },
                value: crate::Value::Str(crate::Str {
                    value: "Hello World".to_string(),
                    span: 9..22,
                }),
                expected_array: false,
            },]),)
        );
    }

    #[test]
    fn multiple_items() {
        assert_eq!(
            config()
                .parse(r#"MyData = "Hello World"; MyOtherData = 1234;"#,)
                .output(),
            Some(&Config(vec![
                crate::Property::Entry {
                    name: crate::Ident {
                        value: "MyData".to_string(),
                        span: 0..6,
                    },
                    value: crate::Value::Str(crate::Str {
                        value: "Hello World".to_string(),
                        span: 9..22,
                    }),
                    expected_array: false,
                },
                crate::Property::Entry {
                    name: crate::Ident {
                        value: "MyOtherData".to_string(),
                        span: 24..35,
                    },
                    value: crate::Value::Number(crate::Number::Int32 {
                        value: 1234,
                        span: 38..42,
                    }),
                    expected_array: false,
                },
            ]),)
        );
    }

    #[test]
    fn class() {
        assert_eq!(
            config()
                .parse(
                    r#"class MyClass {
                    MyData = "Hello World";
                    MyOtherData = 1234;
                };"#,
                )
                .output(),
            Some(&Config(vec![crate::Property::Class(crate::Class::Local {
                name: crate::Ident {
                    value: "MyClass".to_string(),
                    span: 6..13,
                },
                parent: None,
                properties: vec![
                    crate::Property::Entry {
                        name: crate::Ident {
                            value: "MyData".to_string(),
                            span: 36..42,
                        },
                        value: crate::Value::Str(crate::Str {
                            value: "Hello World".to_string(),
                            span: 45..58,
                        }),
                        expected_array: false,
                    },
                    crate::Property::Entry {
                        name: crate::Ident {
                            value: "MyOtherData".to_string(),
                            span: 80..91,
                        },
                        value: crate::Value::Number(crate::Number::Int32 {
                            value: 1234,
                            span: 94..98,
                        }),
                        expected_array: false,
                    },
                ],
                err_missing_braces: false,
            }),]),)
        );
    }

    #[test]
    fn nested_class() {
        assert_eq!(
            config()
                .parse(
                    r#"class Outer {
                    class Inner {
                        MyData = "Hello World";
                        MyOtherData = 1234;
                    };
                };"#,
                )
                .output(),
            Some(&Config(vec![crate::Property::Class(crate::Class::Local {
                err_missing_braces: false,
                name: crate::Ident {
                    value: "Outer".to_string(),
                    span: 6..11,
                },
                parent: None,
                properties: vec![crate::Property::Class(crate::Class::Local {
                    name: crate::Ident {
                        value: "Inner".to_string(),
                        span: 40..45,
                    },
                    parent: None,
                    properties: vec![
                        crate::Property::Entry {
                            name: crate::Ident {
                                value: "MyData".to_string(),
                                span: 72..78,
                            },
                            value: crate::Value::Str(crate::Str {
                                value: "Hello World".to_string(),
                                span: 81..94
                            }),
                            expected_array: false,
                        },
                        crate::Property::Entry {
                            name: crate::Ident {
                                value: "MyOtherData".to_string(),
                                span: 120..131
                            },
                            value: crate::Value::Number(crate::Number::Int32 {
                                value: 1234,
                                span: 134..138
                            }),
                            expected_array: false,
                        },
                    ],
                    err_missing_braces: false,
                })],
            }),]),)
        );
    }
}
