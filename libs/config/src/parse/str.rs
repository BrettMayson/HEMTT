use chumsky::prelude::*;

use crate::Str;

pub fn string<'a>() -> impl Parser<'a, &'a str, Str, extra::Err<Rich<'a, char>>> + Clone {
    let content = none_of('"').or(just(['"'; 2]).to('"'));
    let segment = just('"')
        .ignore_then(content.repeated().collect::<String>())
        .then_ignore(just('"'));
    segment
        .separated_by(just("\\n").padded())
        .at_least(1)
        .collect::<Vec<String>>()
        .map_with(|tok, extra| (tok, extra.span() as SimpleSpan))
        .map(|(segments, span)| Str {
            value: segments
                .into_iter()
                .collect::<Vec<_>>()
                .join("\n")
                .replace("\\\n", ""),
            span: span.into_range(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        assert_eq!(
            string().parse("\"hello world\"").unwrap(),
            Str {
                value: "hello world".to_string(),
                span: 0..13
            }
        );
    }

    #[test]
    fn multiline() {
        assert_eq!(
            string().parse("\"hello\" \\n \"world\"").unwrap(),
            Str {
                value: "hello\nworld".to_string(),
                span: 0..18
            }
        );
    }

    #[test]
    fn escaped() {
        assert_eq!(
            string().parse("\"hello \"\"world\"\"\"").unwrap(),
            Str {
                value: "hello \"world\"".to_string(),
                span: 0..17
            }
        );
        assert_eq!(
            string()
                .parse("\"and he said \"\"hello \"\"\"\"world\"\"\"\"\"\"\"")
                .unwrap(),
            Str {
                value: "and he said \"hello \"\"world\"\"\"".to_string(),
                span: 0..37
            }
        );
    }

    #[test]
    fn multiline_escape() {
        assert_eq!(
            string().parse("\"hello\\\nworld\"").unwrap(),
            Str {
                value: "helloworld".to_string(),
                span: 0..14
            }
        );
        assert_eq!(
            string()
                .parse(
                    r#""\
                'multi';\
                'line';\
            ""#
                )
                .unwrap(),
            Str {
                value: "                'multi';                'line';            ".to_string(),
                span: 0..67
            }
        );
    }
}
