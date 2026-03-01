use std::sync::Arc;

use chumsky::prelude::*;

use crate::{Str, parse::ParseError};

pub fn string<'src>(
    delimiter: char,
) -> impl Parser<'src, &'src str, Str, ParseError<'src>> + Clone {
    let content = (any().and_is(just(delimiter).not())).or(just([delimiter; 2]).to(delimiter));
    let segment = just(delimiter)
        .ignore_then(content.repeated().collect::<String>())
        .then_ignore(just(delimiter));
    segment
        .separated_by(just("\\n").padded())
        .at_least(1)
        .collect::<Vec<String>>()
        .map(|segments: Vec<String>| Str(Arc::from(segments.join("\n").replace("\\\n", ""))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        assert_eq!(
            string('"').parse("\"hello world\"").into_output(),
            Some(Str(Arc::from("hello world")))
        );
    }

    #[test]
    fn multiline() {
        assert_eq!(
            string('"').parse("\"hello\" \\n \"world\"").into_output(),
            Some(Str(Arc::from("hello\nworld")))
        );
    }

    #[test]
    fn escaped() {
        assert_eq!(
            string('"').parse("\"hello \"\"world\"\"\"").into_output(),
            Some(Str(Arc::from("hello \"world\"")))
        );
        assert_eq!(
            string('"')
                .parse("\"and he said \"\"hello \"\"\"\"world\"\"\"\"\"\"\"")
                .into_output(),
            Some(Str(Arc::from("and he said \"hello \"\"world\"\"\"")))
        );
    }

    #[test]
    fn multiline_escape() {
        assert_eq!(
            string('"').parse("\"hello\\\nworld\"").into_output(),
            Some(Str(Arc::from("helloworld")))
        );
        assert_eq!(
            string('"')
                .parse(
                    r#""\
                'multi';\
                'line';\
            ""#
                )
                .into_output(),
            Some(Str(Arc::from(
                "                'multi';                'line';            "
            )))
        );
    }
}
