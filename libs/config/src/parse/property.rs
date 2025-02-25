use chumsky::prelude::*;

use crate::{Class, Property, Value};

use super::{ident::ident, value::value};

fn class_parent<'a>() -> impl Parser<'a, &'a str, crate::Ident, extra::Err<Rich<'a, char>>> + Clone
{
    just(':')
        .padded()
        .ignore_then(ident().padded().labelled("class parent"))
}

fn class_missing_braces<'a>() -> impl Parser<'a, &'a str, Class, extra::Err<Rich<'a, char>>> + Clone
{
    just("class ")
        .padded()
        .ignore_then(ident().padded().labelled("class name"))
        .then(class_parent())
        .padded()
        .map(|(ident, parent)| Class::Local {
            name: ident,
            parent: Some(parent),
            properties: vec![],
            err_missing_braces: true,
        })
}

#[allow(clippy::too_many_lines)]
pub fn property<'a>() -> impl Parser<'a, &'a str, Property, extra::Err<Rich<'a, char>>> + Clone {
    let recovery = none_of(';').repeated().map_with(|(), extra| {
        Value::Invalid((extra.span() as SimpleSpan).into_range())
    });

    let class_external = just("class ")
        .padded()
        .ignore_then(ident().padded().labelled("class name"))
        .padded()
        .map(|ident| Class::External { name: ident });

    recursive(|rec| {
        let properties = rec
            .labelled("class property")
            .padded()
            .repeated()
            .collect::<Vec<Property>>()
            .padded()
            .delimited_by(just('{'), just('}'));

        let class_local = just("class ")
            .padded()
            .ignore_then(ident().padded().labelled("class name"))
            .then(class_parent().or_not())
            .padded()
            .then(properties)
            .map(|((ident, parent), properties)| Class::Local {
                name: ident,
                parent,
                properties,
                err_missing_braces: false,
            });
        let class = choice((class_local, class_missing_braces(), class_external));
        choice((
            class.map(Property::Class),
            just("delete ")
                .padded()
                .ignore_then(ident().labelled("delete class name"))
                .map(Property::Delete),
            ident()
                .labelled("property name")
                .padded()
                .then(
                    just("[]")
                        .padded()
                        .ignore_then(
                            just('=')
                                .padded()
                                .ignore_then(
                                    super::array::array(false)
                                        .map(Value::Array)
                                        .or(value())
                                        .padded()
                                        .labelled("array value")
                                        .recover_with(via_parser(recovery)),
                                )
                                .or(just("+=")
                                    .padded()
                                    .ignore_then(super::array::array(true).map(Value::Array))
                                    .or(value())
                                    .padded()
                                    .labelled("array expand value"))
                                .recover_with(via_parser(recovery))
                                .map(|value| (value, true)),
                        )
                        .or(just('=')
                            .padded()
                            .ignore_then(
                                value()
                                    // .recover_with(via_parser(recovery))
                                    .padded()
                                    .labelled("property value"),
                            )
                            .recover_with(via_parser(recovery))
                            .map(|value| (value, false))),
                )
                .map(|(name, (value, expected_array))| Property::Entry {
                    name,
                    value,
                    expected_array,
                }),
        ))
        .then(just(';').padded().or_not())
        .map_with(|(property, semi), extra| {
            if semi.is_some() {
                property
            } else {
                Property::MissingSemicolon(property.name().clone(), extra.span().into_range())
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use crate::{Str, Value};

    use super::*;

    #[test]
    fn array() {
        assert_eq!(
            property().parse("MyProperty[] = {1,2,3};").output(),
            Some(&Property::Entry {
                name: crate::Ident {
                    value: "MyProperty".to_string(),
                    span: 0..10,
                },
                value: Value::Array(crate::Array {
                    expand: false,
                    items: vec![
                        crate::Item::Number(crate::Number::Int32 {
                            value: 1,
                            span: 16..17,
                        }),
                        crate::Item::Number(crate::Number::Int32 {
                            value: 2,
                            span: 18..19,
                        }),
                        crate::Item::Number(crate::Number::Int32 {
                            value: 3,
                            span: 20..21,
                        }),
                    ],
                    span: 15..22,
                }),
                expected_array: true,
            })
        );
    }

    #[test]
    fn array_expand() {
        assert_eq!(
            property().parse("MyProperty[] += {1,2,3};").output(),
            Some(&Property::Entry {
                name: crate::Ident {
                    value: "MyProperty".to_string(),
                    span: 0..10,
                },
                value: Value::Array(crate::Array {
                    expand: true,
                    items: vec![
                        crate::Item::Number(crate::Number::Int32 {
                            value: 1,
                            span: 17..18
                        }),
                        crate::Item::Number(crate::Number::Int32 {
                            value: 2,
                            span: 19..20
                        }),
                        crate::Item::Number(crate::Number::Int32 {
                            value: 3,
                            span: 21..22
                        }),
                    ],
                    span: 16..23,
                }),
                expected_array: true,
            })
        );
    }

    #[test]
    fn array_empty() {
        assert_eq!(
            property().parse("MyProperty[] = {};").output(),
            Some(&Property::Entry {
                name: crate::Ident {
                    value: "MyProperty".to_string(),
                    span: 0..10,
                },
                value: Value::Array(crate::Array {
                    expand: false,
                    items: vec![],
                    span: 15..17,
                }),
                expected_array: true,
            })
        );
    }

    #[test]
    fn array_nested() {
        assert_eq!(
            property()
                .parse("MyProperty[] = {{1,2,3},{4,5,6}};")
                .output(),
            Some(&Property::Entry {
                name: crate::Ident {
                    value: "MyProperty".to_string(),
                    span: 0..10,
                },
                value: Value::Array(crate::Array {
                    expand: false,
                    items: vec![
                        crate::Item::Array(vec![
                            crate::Item::Number(crate::Number::Int32 {
                                value: 1,
                                span: 17..18
                            }),
                            crate::Item::Number(crate::Number::Int32 {
                                value: 2,
                                span: 19..20
                            }),
                            crate::Item::Number(crate::Number::Int32 {
                                value: 3,
                                span: 21..22
                            }),
                        ]),
                        crate::Item::Array(vec![
                            crate::Item::Number(crate::Number::Int32 {
                                value: 4,
                                span: 25..26,
                            }),
                            crate::Item::Number(crate::Number::Int32 {
                                value: 5,
                                span: 27..28,
                            }),
                            crate::Item::Number(crate::Number::Int32 {
                                value: 6,
                                span: 29..30,
                            }),
                        ]),
                    ],
                    span: 15..32,
                }),
                expected_array: true,
            })
        );
    }

    #[test]
    fn string() {
        assert_eq!(
            property().parse("MyProperty = \"Hello, World!\";").output(),
            Some(&Property::Entry {
                name: crate::Ident {
                    value: "MyProperty".to_string(),
                    span: 0..10,
                },
                value: Value::Str(Str {
                    value: "Hello, World!".to_string(),
                    span: 13..28,
                }),
                expected_array: false,
            })
        );
    }

    #[test]
    fn number() {
        assert_eq!(
            property().parse("MyProperty = 1234;").output(),
            Some(&Property::Entry {
                name: crate::Ident {
                    value: "MyProperty".to_string(),
                    span: 0..10,
                },
                value: Value::Number(crate::Number::Int32 {
                    value: 1234,
                    span: 13..17,
                }),
                expected_array: false,
            })
        );
    }

    #[test]
    fn class_external() {
        use super::*;

        assert_eq!(
            property().parse("class MyClass;").output(),
            Some(&Property::Class(Class::External {
                name: crate::Ident {
                    value: "MyClass".to_string(),
                    span: 6..13,
                }
            }))
        );
    }

    #[test]
    fn class_local() {
        use super::*;

        assert_eq!(
            property()
                .parse("class MyClass { MyProperty = 1; };")
                .output(),
            Some(&Property::Class(Class::Local {
                name: crate::Ident {
                    value: "MyClass".to_string(),
                    span: 6..13,
                },
                parent: None,
                properties: vec![crate::Property::Entry {
                    name: crate::Ident {
                        value: "MyProperty".to_string(),
                        span: 16..26,
                    },
                    value: crate::Value::Number(crate::Number::Int32 {
                        value: 1,
                        span: 29..30,
                    }),
                    expected_array: false,
                }],
                err_missing_braces: false,
            }))
        );
    }

    #[test]
    fn no_whitespace() {
        assert_eq!(
            property().parse("MyProperty=1234;").output(),
            Some(&Property::Entry {
                name: crate::Ident {
                    value: "MyProperty".to_string(),
                    span: 0..10,
                },
                value: Value::Number(crate::Number::Int32 {
                    value: 1234,
                    span: 11..15,
                }),
                expected_array: false,
            })
        );
    }

    #[test]
    fn plenty_whitespace() {
        assert_eq!(
            property().parse("   MyProperty     =      1234;").output(),
            Some(&Property::Entry {
                name: crate::Ident {
                    value: "MyProperty".to_string(),
                    span: 3..13,
                },
                value: Value::Number(crate::Number::Int32 {
                    value: 1234,
                    span: 25..29,
                }),
                expected_array: false,
            })
        );
    }

    #[test]
    fn math() {
        assert_eq!(
            property().parse("math = 1 + 1;").output(),
            Some(&Property::Entry {
                name: crate::Ident {
                    value: "math".to_string(),
                    span: 0..4,
                },
                value: Value::Number(crate::Number::Int32 {
                    value: 2,
                    span: 7..12,
                }),
                expected_array: false,
            })
        );
        println!(
            "{:?}",
            property()
                .parse("math = 1 + one;")
                .errors()
                .collect::<Vec<_>>()
        );
        assert_eq!(
            property().parse("math = 1 + one;").output(),
            Some(&Property::Entry {
                name: crate::Ident {
                    value: "math".to_string(),
                    span: 0..4
                },
                value: Value::Number(crate::Number::InvalidMath { span: 11..14 }),
                expected_array: false
            })
        );
    }

    #[test]
    fn invalid_external_with_parent() {
        assert_eq!(
            property().parse("class MyClass: MyParent;").output(),
            Some(&Property::Class(Class::Local {
                name: crate::Ident {
                    value: "MyClass".to_string(),
                    span: 6..13,
                },
                parent: Some(crate::Ident {
                    value: "MyParent".to_string(),
                    span: 15..23,
                }),
                properties: vec![],
                err_missing_braces: true,
            }))
        );
    }
}
