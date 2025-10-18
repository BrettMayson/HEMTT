use chumsky::prelude::*;

use crate::{Class, Property, Value};

use super::{ident::ident, value::value};

fn class_parent() -> impl Parser<char, crate::Ident, Error = Simple<char>> {
    just(':')
        .padded()
        .ignore_then(ident().padded().labelled("class parent"))
}

fn class_missing_braces() -> impl Parser<char, Class, Error = Simple<char>> {
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
pub fn property() -> impl Parser<char, Property, Error = Simple<char>> {
    recursive(|rec| {
        let properties = just('{')
            .ignore_then(rec.labelled("class property").padded().repeated().padded())
            .then_ignore(just('}').padded().or_not());

        let class_external = just("class ")
            .padded()
            .ignore_then(ident().padded().labelled("class name"))
            .padded()
            .map(|ident| Class::External { name: ident });
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
                                        .recover_with(skip_until([';'], Value::Invalid)),
                                )
                                .or(just("+=")
                                    .padded()
                                    .ignore_then(super::array::array(true).map(Value::Array))
                                    .or(value())
                                    .padded()
                                    .labelled("array expand value"))
                                .recover_with(skip_until([';'], Value::Invalid))
                                .map(|value| (value, true)),
                        )
                        .or(just('=')
                            .padded()
                            .ignore_then(
                                value()
                                    .padded()
                                    .then_ignore(just(';').rewind())
                                    .recover_with(skip_until([';', '\n', '}'], Value::Invalid))
                                    .padded()
                                    .labelled("property value"),
                            )
                            .map(|value| (value, false))),
                )
                .map(|(name, (value, expected_array))| Property::Entry {
                    name,
                    value,
                    expected_array,
                }),
        ))
        .then(just(';').padded().or_not())
        .map_with_span(|(property, semi), range| {
            if semi.is_some() {
                property
            } else {
                Property::MissingSemicolon(property.name().clone(), range)
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
            property().parse("MyProperty[] = {1,2,3};"),
            Ok(Property::Entry {
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
            property().parse("MyProperty[] += {1,2,3};"),
            Ok(Property::Entry {
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
            property().parse("MyProperty[] = {};"),
            Ok(Property::Entry {
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
            property().parse("MyProperty[] = {{1,2,3},{4,5,6}};"),
            Ok(Property::Entry {
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

    // #[test]
    // fn array_nested_missing() {
    //     assert_eq!(
    //         property()
    //             .parse_recovery("MyProperty[] = {{1,2,3},{4,5,6};")
    //             .0,
    //         Some(Property::Entry {
    //             name: crate::Ident {
    //                 value: "MyProperty".to_string(),
    //                 span: 0..10,
    //             },
    //             value: Value::Array(crate::Array {
    //                 expand: false,
    //                 items: vec![
    //                     crate::Item::Array(vec![
    //                         crate::Item::Number(crate::Number::Int32 {
    //                             value: 1,
    //                             span: 0..1
    //                         }),
    //                         crate::Item::Number(crate::Number::Int32 {
    //                             value: 2,
    //                             span: 2..3
    //                         }),
    //                         crate::Item::Number(crate::Number::Int32 {
    //                             value: 3,
    //                             span: 4..5
    //                         }),
    //                     ]),
    //                     crate::Item::Array(vec![
    //                         crate::Item::Number(crate::Number::Int32 {
    //                             value: 4,
    //                             span: 6..7
    //                         }),
    //                         crate::Item::Number(crate::Number::Int32 {
    //                             value: 5,
    //                             span: 8..9
    //                         }),
    //                         crate::Item::Number(crate::Number::Int32 {
    //                             value: 6,
    //                             span: 10..11
    //                         }),
    //                     ]),
    //                 ],
    //                 span: 15..32,
    //             })
    //         })
    //     );
    // }

    #[test]
    fn string() {
        assert_eq!(
            property().parse("MyProperty = \"Hello, World!\";"),
            Ok(Property::Entry {
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
            property().parse("MyProperty = 1234;"),
            Ok(Property::Entry {
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
        assert_eq!(
            property().parse("MyProperty = 1.2;"),
            Ok(Property::Entry {
                name: crate::Ident {
                    value: "MyProperty".to_string(),
                    span: 0..10,
                },
                value: Value::Number(crate::Number::Float32 {
                    value: 1.2,
                    span: 13..16,
                }),
                expected_array: false,
            })
        );
        assert!(property().parse("MyProperty = 1.0.2;").is_err());
        assert!(property().parse("MyProperty = 1 ;").is_ok());
    }

    #[test]
    fn class_external() {
        use super::*;

        assert_eq!(
            property().parse_recovery_verbose("class MyClass;"),
            (
                Some(Property::Class(Class::External {
                    name: crate::Ident {
                        value: "MyClass".to_string(),
                        span: 6..13,
                    }
                })),
                vec![]
            )
        );
    }

    #[test]
    fn class_local() {
        use super::*;

        assert_eq!(
            property().parse_recovery_verbose("class MyClass { MyProperty = 1; };"),
            (
                Some(Property::Class(Class::Local {
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
                })),
                vec![]
            )
        );
    }

    #[test]
    fn no_whitespace() {
        assert_eq!(
            property().parse("MyProperty=1234;"),
            Ok(Property::Entry {
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
            property().parse("   MyProperty     =      1234;"),
            Ok(Property::Entry {
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
            property().parse("math = 1 + 1;"),
            Ok(Property::Entry {
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
        assert_eq!(
            property().parse("math = -0.01*0.5;"),
            Ok(Property::Entry {
                name: crate::Ident {
                    value: "math".to_string(),
                    span: 0..4,
                },
                value: Value::Number(crate::Number::Float32 {
                    value: -0.01 * 0.5,
                    span: 7..16,
                }),
                expected_array: false,
            })
        );
        assert!(property().parse("math = 1 + one;").is_err());
    }

    #[test]
    fn invalid_external_with_parent() {
        assert_eq!(
            property().parse_recovery_verbose("class MyClass: MyParent;"),
            (
                Some(Property::Class(Class::Local {
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
                })),
                vec![]
            )
        );
    }
}
