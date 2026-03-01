use std::sync::Arc;

use chumsky::prelude::*;

use crate::{Array, Item, parse::ParseError};

use super::value::math;

pub fn array<'src>(
    expand: bool,
) -> impl Parser<'src, &'src str, Spanned<Array>, ParseError<'src>> + Clone {
    recursive(|value| {
        value
            .map(Item::Array)
            .spanned()
            .or(array_value().spanned().recover_with(via_parser(
                none_of("},")
                    .padded()
                    .repeated()
                    .at_least(1)
                    .to_slice()
                    .map(|s| Item::Invalid(Arc::from(s)))
                    .spanned(),
            )))
            .padded()
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just('{').padded(), just('}').padded())
    })
    .spanned()
    .map(move |items| Array { expand, items })
    .spanned()
}

fn array_value<'src>() -> impl Parser<'src, &'src str, Item, ParseError<'src>> + Clone {
    choice((
        super::str::string('"').map(Item::Str),
        math().map(Item::Number),
        super::number::number().map(Item::Number),
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::Number;

    use super::*;

    #[test]
    fn empty() {
        assert_eq!(
            array(false).parse("{}").unwrap().inner,
            Array {
                expand: false,
                items: Spanned {
                    inner: vec![],
                    span: SimpleSpan {
                        start: 0,
                        end: 2,
                        context: ()
                    }
                },
            }
        );
    }

    #[test]
    fn single() {
        assert_eq!(
            array(false).parse("{1,2,3}").unwrap().inner,
            Array {
                expand: false,
                items: Spanned {
                    inner: vec![
                        Spanned {
                            inner: Item::Number(Number::Int32(1)),
                            span: SimpleSpan {
                                start: 1,
                                end: 2,
                                context: ()
                            },
                        },
                        Spanned {
                            inner: Item::Number(Number::Int32(2)),
                            span: SimpleSpan {
                                start: 3,
                                end: 4,
                                context: ()
                            },
                        },
                        Spanned {
                            inner: Item::Number(Number::Int32(3)),
                            span: SimpleSpan {
                                start: 5,
                                end: 6,
                                context: ()
                            },
                        },
                    ],
                    span: SimpleSpan {
                        start: 0,
                        end: 7,
                        context: ()
                    },
                },
            }
        );
    }

    #[test]
    fn nested() {
        assert_eq!(
            array(false).parse("{{1,2},{3,4},5}").unwrap().inner,
            Array {
                expand: false,
                items: Spanned {
                    inner: vec![
                        Spanned {
                            inner: Item::Array(vec![
                                Spanned {
                                    inner: Item::Number(Number::Int32(1)),
                                    span: SimpleSpan {
                                        start: 2,
                                        end: 3,
                                        context: ()
                                    },
                                },
                                Spanned {
                                    inner: Item::Number(Number::Int32(2)),
                                    span: SimpleSpan {
                                        start: 4,
                                        end: 5,
                                        context: ()
                                    },
                                },
                            ]),
                            span: SimpleSpan {
                                start: 1,
                                end: 6,
                                context: ()
                            },
                        },
                        Spanned {
                            inner: Item::Array(vec![
                                Spanned {
                                    inner: Item::Number(Number::Int32(3)),
                                    span: SimpleSpan {
                                        start: 8,
                                        end: 9,
                                        context: ()
                                    },
                                },
                                Spanned {
                                    inner: Item::Number(Number::Int32(4)),
                                    span: SimpleSpan {
                                        start: 10,
                                        end: 11,
                                        context: ()
                                    },
                                },
                            ]),
                            span: SimpleSpan {
                                start: 7,
                                end: 12,
                                context: ()
                            },
                        },
                        Spanned {
                            inner: Item::Number(Number::Int32(5)),
                            span: SimpleSpan {
                                start: 13,
                                end: 14,
                                context: ()
                            },
                        },
                    ],
                    span: SimpleSpan {
                        start: 0,
                        end: 15,
                        context: ()
                    },
                },
            }
        );
    }

    #[test]
    fn trailing() {
        assert_eq!(
            array(false).parse("{1,2,3,}").unwrap().inner,
            Array {
                expand: false,
                items: Spanned {
                    inner: vec![
                        Spanned {
                            inner: Item::Number(Number::Int32(1)),
                            span: SimpleSpan {
                                start: 1,
                                end: 2,
                                context: ()
                            },
                        },
                        Spanned {
                            inner: Item::Number(Number::Int32(2)),
                            span: SimpleSpan {
                                start: 3,
                                end: 4,
                                context: ()
                            },
                        },
                        Spanned {
                            inner: Item::Number(Number::Int32(3)),
                            span: SimpleSpan {
                                start: 5,
                                end: 6,
                                context: ()
                            },
                        },
                    ],
                    span: SimpleSpan {
                        start: 0,
                        end: 8,
                        context: ()
                    },
                },
            }
        );
    }

    #[test]
    fn invalid_item() {
        assert_eq!(
            array(false)
                .parse("{1,2,three,4}")
                .into_output()
                .map(|a| a.inner),
            Some(Array {
                expand: false,
                items: Spanned {
                    inner: vec![
                        Spanned {
                            inner: Item::Number(Number::Int32(1)),
                            span: SimpleSpan {
                                start: 1,
                                end: 2,
                                context: ()
                            },
                        },
                        Spanned {
                            inner: Item::Number(Number::Int32(2)),
                            span: SimpleSpan {
                                start: 3,
                                end: 4,
                                context: ()
                            },
                        },
                        Spanned {
                            inner: Item::Invalid(Arc::from("three")),
                            span: SimpleSpan {
                                start: 5,
                                end: 10,
                                context: ()
                            },
                        },
                        Spanned {
                            inner: Item::Number(Number::Int32(4)),
                            span: SimpleSpan {
                                start: 11,
                                end: 12,
                                context: ()
                            },
                        },
                    ],
                    span: SimpleSpan {
                        start: 0,
                        end: 13,
                        context: ()
                    },
                },
            })
        );
    }
}
