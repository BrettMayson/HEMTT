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

fn raise_span<O, U, W: Fn(O) -> U>(wrapped: Spanned<O>, wrap: W) -> Spanned<U> {
    Spanned {
        inner: wrap(wrapped.inner),
        span: wrapped.span,
    }
}
pub type ParseError<'src> = chumsky::extra::Err<chumsky::error::Rich<'src, char>>;

/// Parse a config file.
pub fn config<'src>() -> impl Parser<'src, &'src str, Config, ParseError<'src>> {
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
    use std::sync::Arc;

    use chumsky::{
        Parser,
        span::{SimpleSpan, Spanned},
    };

    use crate::{Class, Config, Ident, Number, Property, Str, Value, parse::config};

    #[test]
    fn empty() {
        assert_eq!(config().parse(r"",).unwrap(), Config(vec![]));
        assert_eq!(config().parse(r"   ",).unwrap(), Config(vec![]));
    }

    #[test]
    fn single_item() {
        assert_eq!(
            config().parse(r#"MyData = "Hello World";"#,).unwrap(),
            Config(vec![Spanned {
                inner: crate::Property::Entry {
                    name: Spanned {
                        inner: crate::Ident(Arc::from("MyData")),
                        span: SimpleSpan {
                            start: 0,
                            end: 6,
                            context: ()
                        },
                    },
                    value: Spanned {
                        inner: crate::Value::Str(crate::Str(Arc::from("Hello World"))),
                        span: SimpleSpan {
                            start: 9,
                            end: 22,
                            context: ()
                        },
                    },
                    expected_array: false,
                },
                span: SimpleSpan {
                    start: 0,
                    end: 22,
                    context: ()
                },
            },]),
        );
    }

    #[test]
    fn multiple_items() {
        assert_eq!(
            config()
                .parse(r#"MyData = "Hello World"; MyOtherData = 1234;"#,)
                .unwrap(),
            Config(vec![
                Spanned {
                    inner: crate::Property::Entry {
                        name: Spanned {
                            inner: crate::Ident(Arc::from("MyData")),
                            span: SimpleSpan {
                                start: 0,
                                end: 6,
                                context: ()
                            },
                        },
                        value: Spanned {
                            inner: crate::Value::Str(crate::Str(Arc::from("Hello World"))),
                            span: SimpleSpan {
                                start: 9,
                                end: 22,
                                context: ()
                            },
                        },
                        expected_array: false,
                    },
                    span: SimpleSpan {
                        start: 0,
                        end: 22,
                        context: ()
                    },
                },
                Spanned {
                    inner: crate::Property::Entry {
                        name: Spanned {
                            inner: crate::Ident(Arc::from("MyOtherData")),
                            span: SimpleSpan {
                                start: 24,
                                end: 35,
                                context: ()
                            },
                        },
                        value: Spanned {
                            inner: crate::Value::Number(crate::Number::Int32(1234)),
                            span: SimpleSpan {
                                start: 38,
                                end: 42,
                                context: ()
                            },
                        },
                        expected_array: false,
                    },
                    span: SimpleSpan {
                        start: 24,
                        end: 42,
                        context: ()
                    },
                },
            ]),
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
                .unwrap(),
            Config(vec![Spanned {
                inner: crate::Property::Class(Spanned {
                    inner: crate::Class::Local {
                        name: Spanned {
                            inner: crate::Ident(Arc::from("MyClass")),
                            span: SimpleSpan {
                                start: 6,
                                end: 13,
                                context: ()
                            },
                        },
                        parent: None,
                        properties: vec![
                            Spanned {
                                inner: crate::Property::Entry {
                                    name: Spanned {
                                        inner: crate::Ident(Arc::from("MyData")),
                                        span: SimpleSpan {
                                            start: 36,
                                            end: 42,
                                            context: ()
                                        },
                                    },
                                    value: Spanned {
                                        inner: crate::Value::Str(crate::Str(Arc::from(
                                            "Hello World"
                                        ))),
                                        span: SimpleSpan {
                                            start: 45,
                                            end: 58,
                                            context: ()
                                        },
                                    },
                                    expected_array: false,
                                },
                                span: SimpleSpan {
                                    start: 36,
                                    end: 58,
                                    context: ()
                                },
                            },
                            Spanned {
                                inner: crate::Property::Entry {
                                    name: Spanned {
                                        inner: crate::Ident(Arc::from("MyOtherData")),
                                        span: SimpleSpan {
                                            start: 80,
                                            end: 91,
                                            context: ()
                                        },
                                    },
                                    value: Spanned {
                                        inner: crate::Value::Number(crate::Number::Int32(1234)),
                                        span: SimpleSpan {
                                            start: 94,
                                            end: 98,
                                            context: ()
                                        },
                                    },
                                    expected_array: false,
                                },
                                span: SimpleSpan {
                                    start: 80,
                                    end: 98,
                                    context: ()
                                },
                            },
                        ],
                        err_missing_braces: false,
                    },
                    span: SimpleSpan {
                        start: 0,
                        end: 117,
                        context: ()
                    },
                }),
                span: SimpleSpan {
                    start: 0,
                    end: 117,
                    context: ()
                },
            }]),
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
                .unwrap(),
            Config(vec![Spanned {
                inner: Property::Class(Spanned {
                    inner: Class::Local {
                        name: Spanned {
                            inner: Ident(Arc::from("Outer")),
                            // span: 6..11
                            span: SimpleSpan {
                                start: 6,
                                end: 11,
                                context: ()
                            }
                        },
                        parent: None,
                        properties: vec![Spanned {
                            inner: Property::Class(Spanned {
                                inner: Class::Local {
                                    name: Spanned {
                                        inner: Ident(Arc::from("Inner")),
                                        span: SimpleSpan {
                                            start: 40,
                                            end: 45,
                                            context: ()
                                        }
                                    },
                                    parent: None,
                                    properties: vec![
                                        Spanned {
                                            inner: Property::Entry {
                                                name: Spanned {
                                                    inner: Ident(Arc::from("MyData")),
                                                    span: SimpleSpan {
                                                        start: 72,
                                                        end: 78,
                                                        context: ()
                                                    }
                                                },
                                                value: Spanned {
                                                    inner: Value::Str(Str(Arc::from(
                                                        "Hello World"
                                                    ))),
                                                    span: SimpleSpan {
                                                        start: 81,
                                                        end: 94,
                                                        context: ()
                                                    }
                                                },
                                                expected_array: false
                                            },
                                            span: SimpleSpan {
                                                start: 72,
                                                end: 94,
                                                context: ()
                                            }
                                        },
                                        Spanned {
                                            inner: Property::Entry {
                                                name: Spanned {
                                                    inner: Ident(Arc::from("MyOtherData")),
                                                    span: SimpleSpan {
                                                        start: 120,
                                                        end: 131,
                                                        context: ()
                                                    }
                                                },
                                                value: Spanned {
                                                    inner: Value::Number(Number::Int32(1234)),
                                                    span: SimpleSpan {
                                                        start: 134,
                                                        end: 138,
                                                        context: ()
                                                    }
                                                },
                                                expected_array: false
                                            },
                                            span: SimpleSpan {
                                                start: 120,
                                                end: 138,
                                                context: ()
                                            }
                                        }
                                    ],
                                    err_missing_braces: false
                                },
                                span: SimpleSpan {
                                    start: 34,
                                    end: 161,
                                    context: ()
                                }
                            }),
                            span: SimpleSpan {
                                start: 34,
                                end: 161,
                                context: ()
                            }
                        }],
                        err_missing_braces: false
                    },
                    span: SimpleSpan {
                        start: 0,
                        end: 180,
                        context: ()
                    }
                }),
                span: SimpleSpan {
                    start: 0,
                    end: 180,
                    context: ()
                }
            }]),
        );
    }
}
