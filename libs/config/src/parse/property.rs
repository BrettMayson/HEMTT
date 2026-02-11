use chumsky::prelude::*;

use crate::{
    Class, Property, Value,
    parse::{ParseError, raise_span},
};

use super::{ident::ident, value::value};

fn class_parent<'src>()
-> impl Parser<'src, &'src str, Spanned<crate::Ident>, ParseError<'src>> + Clone {
    just(':')
        .padded()
        .ignore_then(ident().padded().labelled("class parent"))
}

fn class_missing_braces<'src>()
-> impl Parser<'src, &'src str, Spanned<Class>, ParseError<'src>> + Clone {
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
        .spanned()
}

pub fn property<'src>() -> impl Parser<'src, &'src str, Spanned<Property>, ParseError<'src>> + Clone
{
    recursive(|rec| {
        let properties = just('{')
            .ignore_then(
                rec.labelled("class property")
                    .padded()
                    .repeated()
                    .collect::<Vec<_>>()
                    .padded(),
            )
            .then_ignore(just('}').padded().or_not());

        let class_external = just("class ")
            .padded()
            .ignore_then(ident().padded().labelled("class name"))
            .padded()
            .map(|ident| Class::External { name: ident })
            .spanned();
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
            })
            .spanned();
        let class = choice((class_local, class_missing_braces(), class_external));
        choice((
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
                                            .map(|a| raise_span(a, Value::Array))
                                            .or(value())
                                            .padded()
                                            .labelled("array value")
                                            .recover_with(via_parser(
                                                none_of(";")
                                                    .repeated()
                                                    .at_least(1)
                                                    .collect::<String>()
                                                    .map(|s: String| Value::Invalid(s.into()))
                                                    .spanned(),
                                            )),
                                    )
                                    .or(just("+=")
                                        .padded()
                                        .ignore_then(
                                            super::array::array(true)
                                                .map(|a| raise_span(a, Value::Array)),
                                        )
                                        .or(value())
                                        .padded()
                                        .labelled("array expand value"))
                                    .recover_with(via_parser(
                                        none_of(";")
                                            .repeated()
                                            .at_least(1)
                                            .collect::<String>()
                                            .map(|s: String| Value::Invalid(s.into()))
                                            .spanned(),
                                    ))
                                    .map(|value| (value, true)),
                            )
                            .or(just('=')
                                .padded()
                                .ignore_then(
                                    value()
                                        .padded()
                                        .then_ignore(just(';').rewind())
                                        .recover_with(via_parser(
                                            none_of(";\n}")
                                                .repeated()
                                                .at_least(1)
                                                .collect::<String>()
                                                .map(|s: String| Value::Invalid(s.into()))
                                                .spanned(),
                                        ))
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
            .spanned()
            .then(just(';').padded().or_not())
            .map(|(property, semi)| {
                if semi.is_some() {
                    property
                } else {
                    Spanned {
                        inner: Property::MissingSemicolon(
                            property.name().expect("valid property").clone(),
                        ),
                        span: property.span,
                    }
                }
            }),
            just(';')
                .repeated()
                .at_least(1)
                .padded()
                .map_with(|(), e| {
                    let span: SimpleSpan = e.span();
                    Property::ExtraSemicolons(SimpleSpan {
                        start: span.start,
                        end: span.end - 1,
                        context: span.context,
                    })
                })
                .spanned(),
        ))
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{Str, Value};

    use super::*;

    #[test]
    fn array() {
        assert_eq!(
            property().parse("MyProperty[] = {1,2,3};").unwrap(),
            Spanned {
                inner: Property::Entry {
                    name: Spanned {
                        inner: crate::Ident(Arc::from("MyProperty")),
                        span: SimpleSpan {
                            start: 0,
                            end: 10,
                            context: ()
                        }
                    },
                    value: Spanned {
                        inner: Value::Array(crate::Array {
                            expand: false,
                            items: Spanned {
                                inner: vec![
                                    Spanned {
                                        inner: crate::Item::Number(crate::Number::Int32(1)),
                                        span: SimpleSpan {
                                            start: 16,
                                            end: 17,
                                            context: ()
                                        }
                                    },
                                    Spanned {
                                        inner: crate::Item::Number(crate::Number::Int32(2)),
                                        span: SimpleSpan {
                                            start: 18,
                                            end: 19,
                                            context: ()
                                        }
                                    },
                                    Spanned {
                                        inner: crate::Item::Number(crate::Number::Int32(3)),
                                        span: SimpleSpan {
                                            start: 20,
                                            end: 21,
                                            context: ()
                                        }
                                    },
                                ],
                                span: SimpleSpan {
                                    start: 15,
                                    end: 22,
                                    context: ()
                                },
                            },
                        }),
                        span: SimpleSpan {
                            start: 15,
                            end: 22,
                            context: ()
                        },
                    },
                    expected_array: true,
                },
                span: SimpleSpan {
                    start: 0,
                    end: 22,
                    context: ()
                },
            }
        );
    }

    #[test]
    fn array_expand() {
        assert_eq!(
            property().parse("MyProperty[] += {1,2,3};").unwrap(),
            Spanned {
                inner: Property::Entry {
                    name: Spanned {
                        inner: crate::Ident(Arc::from("MyProperty")),
                        span: SimpleSpan {
                            start: 0,
                            end: 10,
                            context: ()
                        }
                    },
                    value: Spanned {
                        inner: Value::Array(crate::Array {
                            expand: true,
                            items: Spanned {
                                inner: vec![
                                    Spanned {
                                        inner: crate::Item::Number(crate::Number::Int32(1)),
                                        span: SimpleSpan {
                                            start: 17,
                                            end: 18,
                                            context: ()
                                        }
                                    },
                                    Spanned {
                                        inner: crate::Item::Number(crate::Number::Int32(2)),
                                        span: SimpleSpan {
                                            start: 19,
                                            end: 20,
                                            context: ()
                                        }
                                    },
                                    Spanned {
                                        inner: crate::Item::Number(crate::Number::Int32(3)),
                                        span: SimpleSpan {
                                            start: 21,
                                            end: 22,
                                            context: ()
                                        }
                                    },
                                ],
                                span: SimpleSpan {
                                    start: 16,
                                    end: 23,
                                    context: ()
                                },
                            },
                        }),
                        span: SimpleSpan {
                            start: 16,
                            end: 23,
                            context: ()
                        },
                    },
                    expected_array: true,
                },
                span: SimpleSpan {
                    start: 0,
                    end: 23,
                    context: ()
                },
            }
        );
    }

    #[test]
    fn array_empty() {
        assert_eq!(
            property().parse("MyProperty[] = {};").unwrap(),
            Spanned {
                inner: Property::Entry {
                    name: Spanned {
                        inner: crate::Ident(Arc::from("MyProperty")),
                        span: SimpleSpan {
                            start: 0,
                            end: 10,
                            context: ()
                        }
                    },
                    value: Spanned {
                        inner: Value::Array(crate::Array {
                            expand: false,
                            items: Spanned {
                                inner: vec![],
                                span: SimpleSpan {
                                    start: 15,
                                    end: 17,
                                    context: ()
                                },
                            },
                        }),
                        span: SimpleSpan {
                            start: 15,
                            end: 17,
                            context: ()
                        },
                    },
                    expected_array: true,
                },
                span: SimpleSpan {
                    start: 0,
                    end: 17,
                    context: ()
                },
            }
        );
    }

    #[test]
    fn array_nested() {
        assert_eq!(
            property()
                .parse("MyProperty[] = {{1,2,3},{4,5,6}};")
                .unwrap(),
            Spanned {
                inner: Property::Entry {
                    name: Spanned {
                        inner: crate::Ident(Arc::from("MyProperty")),
                        span: SimpleSpan {
                            start: 0,
                            end: 10,
                            context: ()
                        }
                    },
                    value: Spanned {
                        inner: Value::Array(crate::Array {
                            expand: false,
                            items: Spanned {
                                inner: vec![
                                    Spanned {
                                        inner: crate::Item::Array(vec![
                                            Spanned {
                                                inner: crate::Item::Number(crate::Number::Int32(1)),
                                                span: SimpleSpan {
                                                    start: 17,
                                                    end: 18,
                                                    context: ()
                                                }
                                            },
                                            Spanned {
                                                inner: crate::Item::Number(crate::Number::Int32(2)),
                                                span: SimpleSpan {
                                                    start: 19,
                                                    end: 20,
                                                    context: ()
                                                }
                                            },
                                            Spanned {
                                                inner: crate::Item::Number(crate::Number::Int32(3)),
                                                span: SimpleSpan {
                                                    start: 21,
                                                    end: 22,
                                                    context: ()
                                                }
                                            },
                                        ]),
                                        span: SimpleSpan {
                                            start: 16,
                                            end: 23,
                                            context: ()
                                        }
                                    },
                                    Spanned {
                                        inner: crate::Item::Array(vec![
                                            Spanned {
                                                inner: crate::Item::Number(crate::Number::Int32(4)),
                                                span: SimpleSpan {
                                                    start: 25,
                                                    end: 26,
                                                    context: ()
                                                }
                                            },
                                            Spanned {
                                                inner: crate::Item::Number(crate::Number::Int32(5)),
                                                span: SimpleSpan {
                                                    start: 27,
                                                    end: 28,
                                                    context: ()
                                                }
                                            },
                                            Spanned {
                                                inner: crate::Item::Number(crate::Number::Int32(6)),
                                                span: SimpleSpan {
                                                    start: 29,
                                                    end: 30,
                                                    context: ()
                                                }
                                            },
                                        ]),
                                        span: SimpleSpan {
                                            start: 24,
                                            end: 31,
                                            context: ()
                                        }
                                    },
                                ],
                                span: SimpleSpan {
                                    start: 15,
                                    end: 32,
                                    context: ()
                                },
                            },
                        }),
                        span: SimpleSpan {
                            start: 15,
                            end: 32,
                            context: ()
                        },
                    },
                    expected_array: true,
                },
                span: SimpleSpan {
                    start: 0,
                    end: 32,
                    context: ()
                },
            }
        );
    }

    #[test]
    fn array_nested_missing() {
        let result = property()
            .parse("MyProperty[] = {{1,2,3},{4,5,6};")
            .into_output()
            .expect("Failed to parse property");
        assert_eq!(
            result,
            Spanned {
                inner: Property::Entry {
                    expected_array: true,
                    name: Spanned {
                        inner: crate::Ident(Arc::from("MyProperty")),
                        span: SimpleSpan {
                            start: 0,
                            end: 10,
                            context: ()
                        }
                    },
                    value: Spanned {
                        inner: Value::Invalid(Arc::from("{{1,2,3},{4,5,6}")),
                        span: SimpleSpan {
                            start: 15,
                            end: 31,
                            context: ()
                        },
                    },
                },
                span: SimpleSpan {
                    start: 0,
                    end: 31,
                    context: ()
                },
            }
        );
    }

    #[test]
    fn string() {
        assert_eq!(
            property().parse("MyProperty = \"Hello, World!\";").unwrap(),
            Spanned {
                inner: Property::Entry {
                    name: Spanned {
                        inner: crate::Ident(Arc::from("MyProperty")),
                        span: SimpleSpan {
                            start: 0,
                            end: 10,
                            context: ()
                        }
                    },
                    value: Spanned {
                        inner: Value::Str(Str(Arc::from("Hello, World!"))),
                        span: SimpleSpan {
                            start: 13,
                            end: 28,
                            context: ()
                        },
                    },
                    expected_array: false,
                },
                span: SimpleSpan {
                    start: 0,
                    end: 28,
                    context: ()
                },
            }
        );
    }

    #[test]
    fn number() {
        assert_eq!(
            property().parse("MyProperty = 1234;").unwrap(),
            Spanned {
                inner: Property::Entry {
                    name: Spanned {
                        inner: crate::Ident(Arc::from("MyProperty")),
                        span: SimpleSpan {
                            start: 0,
                            end: 10,
                            context: ()
                        }
                    },
                    value: Spanned {
                        inner: Value::Number(crate::Number::Int32(1234)),
                        span: SimpleSpan {
                            start: 13,
                            end: 17,
                            context: ()
                        },
                    },
                    expected_array: false,
                },
                span: SimpleSpan {
                    start: 0,
                    end: 17,
                    context: ()
                },
            }
        );
        assert_eq!(
            property().parse("MyProperty = 1.2;").unwrap(),
            Spanned {
                inner: Property::Entry {
                    name: Spanned {
                        inner: crate::Ident(Arc::from("MyProperty")),
                        span: SimpleSpan {
                            start: 0,
                            end: 10,
                            context: ()
                        }
                    },
                    value: Spanned {
                        inner: Value::Number(crate::Number::Float32(1.2)),
                        span: SimpleSpan {
                            start: 13,
                            end: 16,
                            context: ()
                        },
                    },
                    expected_array: false,
                },
                span: SimpleSpan {
                    start: 0,
                    end: 16,
                    context: ()
                },
            }
        );
        assert!(property().parse("MyProperty = 1.0.2;").has_errors());
        property().parse("MyProperty = 1 ;").unwrap();
    }

    #[test]
    fn class_external() {
        use super::*;

        let result = property().parse("class MyClass;").unwrap();
        assert_eq!(
            result,
            Spanned {
                inner: Property::Class(Spanned {
                    inner: Class::External {
                        name: Spanned {
                            inner: crate::Ident(Arc::from("MyClass")),
                            span: SimpleSpan {
                                start: 6,
                                end: 13,
                                context: ()
                            }
                        }
                    },
                    span: SimpleSpan {
                        start: 0,
                        end: 13,
                        context: ()
                    },
                }),
                span: SimpleSpan {
                    start: 0,
                    end: 13,
                    context: ()
                },
            }
        );
    }

    #[test]
    fn class_local() {
        use super::*;

        let result = property()
            .parse("class MyClass { MyProperty = 1; };")
            .unwrap();
        assert_eq!(
            result,
            Spanned {
                inner: Property::Class(Spanned {
                    inner: Class::Local {
                        name: Spanned {
                            inner: crate::Ident(Arc::from("MyClass")),
                            span: SimpleSpan {
                                start: 6,
                                end: 13,
                                context: ()
                            }
                        },
                        parent: None,
                        properties: vec![Spanned {
                            inner: crate::Property::Entry {
                                name: Spanned {
                                    inner: crate::Ident(Arc::from("MyProperty")),
                                    span: SimpleSpan {
                                        start: 16,
                                        end: 26,
                                        context: ()
                                    }
                                },
                                value: Spanned {
                                    inner: crate::Value::Number(crate::Number::Int32(1)),
                                    span: SimpleSpan {
                                        start: 29,
                                        end: 30,
                                        context: ()
                                    },
                                },
                                expected_array: false,
                            },
                            span: SimpleSpan {
                                start: 16,
                                end: 30,
                                context: ()
                            },
                        }],
                        err_missing_braces: false,
                    },
                    span: SimpleSpan {
                        start: 0,
                        end: 33,
                        context: ()
                    },
                }),
                span: SimpleSpan {
                    start: 0,
                    end: 33,
                    context: ()
                },
            }
        );
    }

    #[test]
    fn no_whitespace() {
        assert_eq!(
            property().parse("MyProperty=1234;").unwrap(),
            Spanned {
                inner: Property::Entry {
                    name: Spanned {
                        inner: crate::Ident(Arc::from("MyProperty")),
                        span: SimpleSpan {
                            start: 0,
                            end: 10,
                            context: ()
                        }
                    },
                    value: Spanned {
                        inner: Value::Number(crate::Number::Int32(1234)),
                        span: SimpleSpan {
                            start: 11,
                            end: 15,
                            context: ()
                        },
                    },
                    expected_array: false,
                },
                span: SimpleSpan {
                    start: 0,
                    end: 15,
                    context: ()
                },
            }
        );
    }

    #[test]
    fn plenty_whitespace() {
        assert_eq!(
            property().parse("   MyProperty     =      1234;").unwrap(),
            Spanned {
                inner: Property::Entry {
                    name: Spanned {
                        inner: crate::Ident(Arc::from("MyProperty")),
                        span: SimpleSpan {
                            start: 3,
                            end: 13,
                            context: ()
                        }
                    },
                    value: Spanned {
                        inner: Value::Number(crate::Number::Int32(1234)),
                        span: SimpleSpan {
                            start: 25,
                            end: 29,
                            context: ()
                        },
                    },
                    expected_array: false,
                },
                span: SimpleSpan {
                    start: 0,
                    end: 29,
                    context: ()
                },
            }
        );
    }

    #[test]
    fn math() {
        assert_eq!(
            property().parse("math = 1 + 1;").unwrap(),
            Spanned {
                inner: Property::Entry {
                    name: Spanned {
                        inner: crate::Ident(Arc::from("math")),
                        span: SimpleSpan {
                            start: 0,
                            end: 4,
                            context: ()
                        }
                    },
                    value: Spanned {
                        inner: Value::Number(crate::Number::Int32(2)),
                        span: SimpleSpan {
                            start: 7,
                            end: 12,
                            context: ()
                        },
                    },
                    expected_array: false,
                },
                span: SimpleSpan {
                    start: 0,
                    end: 12,
                    context: ()
                },
            }
        );
        assert_eq!(
            property().parse("math = -0.01*0.5;").unwrap(),
            Spanned {
                inner: Property::Entry {
                    name: Spanned {
                        inner: crate::Ident(Arc::from("math")),
                        span: SimpleSpan {
                            start: 0,
                            end: 4,
                            context: ()
                        }
                    },
                    value: Spanned {
                        inner: Value::Number(crate::Number::Float32(-0.01 * 0.5)),
                        span: SimpleSpan {
                            start: 7,
                            end: 16,
                            context: ()
                        },
                    },
                    expected_array: false,
                },
                span: SimpleSpan {
                    start: 0,
                    end: 16,
                    context: ()
                },
            }
        );
        assert!(property().parse("math = 1 + one;").has_errors());
    }

    #[test]
    fn invalid_external_with_parent() {
        let result = property().parse("class MyClass: MyParent;").unwrap();
        assert_eq!(
            result,
            Spanned {
                inner: Property::Class(Spanned {
                    inner: Class::Local {
                        name: Spanned {
                            inner: crate::Ident(Arc::from("MyClass")),
                            span: SimpleSpan {
                                start: 6,
                                end: 13,
                                context: ()
                            }
                        },
                        parent: Some(Spanned {
                            inner: crate::Ident(Arc::from("MyParent")),
                            span: SimpleSpan {
                                start: 15,
                                end: 23,
                                context: ()
                            }
                        }),
                        properties: vec![],
                        err_missing_braces: true,
                    },
                    span: SimpleSpan {
                        start: 0,
                        end: 23,
                        context: ()
                    },
                }),
                span: SimpleSpan {
                    start: 0,
                    end: 23,
                    context: ()
                },
            }
        );
    }
}
