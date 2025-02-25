use chumsky::prelude::*;

use crate::{Array, Item};

use super::value::math;

pub fn array<'a>(
    expand: bool,
) -> impl Parser<'a, &'a str, Array, extra::Err<Rich<'a, char>>> + Clone {
    let recovery = none_of("},")
        .padded()
        .repeated()
        .at_least(1)
        .map_with(move |(), extra| Item::Invalid((extra.span() as SimpleSpan).into_range()));
    recursive(|value| {
        value
            .map(Item::Array)
            .or(array_value().recover_with(via_parser(recovery)))
            .padded()
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect()
            .delimited_by(just('{').padded(), just('}').padded())
    })
    .map_with(move |items, extra| Array {
        expand,
        items,
        span: extra.span().into_range(),
    })
}

fn array_value<'a>() -> impl Parser<'a, &'a str, Item, extra::Err<Rich<'a, char>>> + Clone {
    choice((
        super::str::string().map(Item::Str),
        math().map(Item::Number),
        super::number::number().map(Item::Number),
    ))
}

#[cfg(test)]
mod tests {
    use crate::Number;

    use super::*;

    #[test]
    fn empty() {
        assert_eq!(
            array(false).parse("{}").output(),
            Some(&Array {
                expand: false,
                items: vec![],
                span: 0..2,
            })
        );
    }

    #[test]
    fn single() {
        assert_eq!(
            array(false).parse("{1,2,3}").output(),
            Some(&Array {
                expand: false,
                items: vec![
                    Item::Number(Number::Int32 {
                        value: 1,
                        span: 1..2,
                    }),
                    Item::Number(Number::Int32 {
                        value: 2,
                        span: 3..4,
                    }),
                    Item::Number(Number::Int32 {
                        value: 3,
                        span: 5..6,
                    }),
                ],
                span: 0..7,
            })
        );
    }

    #[test]
    fn nested() {
        assert_eq!(
            array(false).parse("{{1,2},{3,4},5}").output(),
            Some(&Array {
                expand: false,
                items: vec![
                    Item::Array(vec![
                        Item::Number(Number::Int32 {
                            value: 1,
                            span: 2..3
                        }),
                        Item::Number(Number::Int32 {
                            value: 2,
                            span: 4..5
                        }),
                    ]),
                    Item::Array(vec![
                        Item::Number(Number::Int32 {
                            value: 3,
                            span: 8..9
                        }),
                        Item::Number(Number::Int32 {
                            value: 4,
                            span: 10..11
                        }),
                    ]),
                    Item::Number(Number::Int32 {
                        value: 5,
                        span: 13..14
                    }),
                ],
                span: 0..15
            })
        );
    }

    #[test]
    fn trailing() {
        assert_eq!(
            array(false).parse("{1,2,3,}").output(),
            Some(&Array {
                expand: false,
                items: vec![
                    Item::Number(Number::Int32 {
                        value: 1,
                        span: 1..2,
                    }),
                    Item::Number(Number::Int32 {
                        value: 2,
                        span: 3..4,
                    }),
                    Item::Number(Number::Int32 {
                        value: 3,
                        span: 5..6,
                    }),
                ],
                span: 0..8,
            })
        );
    }

    #[test]
    fn invalid_item() {
        assert_eq!(
            array(false).parse("{1,2,three,4}").output(),
            Some(&Array {
                expand: false,
                items: vec![
                    Item::Number(Number::Int32 {
                        value: 1,
                        span: 1..2,
                    }),
                    Item::Number(Number::Int32 {
                        value: 2,
                        span: 3..4,
                    }),
                    Item::Invalid(5..10),
                    Item::Number(Number::Int32 {
                        value: 4,
                        span: 11..12,
                    }),
                ],
                span: 0..13,
            })
        );
    }
}
