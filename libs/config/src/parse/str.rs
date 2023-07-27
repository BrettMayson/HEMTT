use chumsky::prelude::*;

use crate::Str;

pub fn string(delimiter: char) -> impl Parser<char, Str, Error = Simple<char>> {
    let content = just(delimiter).not().or(just([delimiter; 2]).to(delimiter));
    let segment = just(delimiter)
        .ignore_then(content.repeated())
        .then_ignore(just(delimiter))
        .collect();
    segment
        .separated_by(just("\\n").padded())
        .at_least(1)
        .collect::<Vec<String>>()
        .map_with_span(|tok, span| (tok, span))
        .map(|(segments, span)| Str {
            value: segments
                .into_iter()
                .collect::<Vec<_>>()
                .join("\n")
                .replace("\\\n", ""),
            span,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        assert_eq!(
            string('"').parse("\"hello world\""),
            Ok(Str {
                value: "hello world".to_string(),
                span: 0..13
            })
        );
    }

    #[test]
    fn multiline() {
        assert_eq!(
            string('"').parse("\"hello\" \\n \"world\""),
            Ok(Str {
                value: "hello\nworld".to_string(),
                span: 0..18
            })
        );
    }

    #[test]
    fn escaped() {
        assert_eq!(
            string('"').parse("\"hello \"\"world\"\"\""),
            Ok(Str {
                value: "hello \"world\"".to_string(),
                span: 0..17
            })
        );
        assert_eq!(
            string('"').parse("\"and he said \"\"hello \"\"\"\"world\"\"\"\"\"\"\""),
            Ok(Str {
                value: "and he said \"hello \"\"world\"\"\"".to_string(),
                span: 0..37
            })
        );
    }

    #[test]
    fn multiline_escape() {
        assert_eq!(
            string('"').parse("\"hello\\\nworld\""),
            Ok(Str {
                value: "helloworld".to_string(),
                span: 0..14
            })
        );
        assert_eq!(
            string('"').parse(
                r#""\
                'multi';\
                'line';\
            ""#
            ),
            Ok(Str {
                value: "                'multi';                'line';            ".to_string(),
                span: 0..67
            })
        );
    }
}
