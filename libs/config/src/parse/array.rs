use chumsky::prelude::*;

use crate::{Array, Item};

pub fn array(expand: bool) -> impl Parser<char, Array, Error = Simple<char>> {
    recursive(|value| {
        value
            .map(Item::Array)
            .or(array_value())
            .recover_with(skip_then_retry_until([',', '}', ';']))
            .separated_by(just(',').padded())
            .delimited_by(just('{'), just('}'))
    })
    .map_with_span(move |items, span| Array {
        expand,
        items,
        span,
    })
}

fn array_value() -> impl Parser<char, Item, Error = Simple<char>> {
    choice((
        super::str::string('"').map(Item::Str),
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
            array(false).parse("{}"),
            Ok(Array {
                expand: false,
                items: vec![],
                span: 0..2,
            })
        );
    }

    #[test]
    fn single() {
        assert_eq!(
            array(false).parse("{1,2,3}"),
            Ok(Array {
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
            array(false).parse("{{1,2},{3,4},5}"),
            Ok(Array {
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
}
