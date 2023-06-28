use chumsky::prelude::*;

use crate::{Class, Property};

use super::{ident::ident, property::property};

pub fn class() -> impl Parser<char, Class, Error = Simple<char>> {
    just("class")
        .padded()
        .ignore_then(ident().padded().labelled("class name"))
        .then(
            (just(':')
                .padded()
                .ignore_then(ident().padded().labelled("class parent")))
            .or_not(),
        )
        .padded()
        .then(
            property()
                .labelled("class property")
                .padded()
                .repeated()
                .delimited_by(just('{'), just('}'))
                .padded()
                .map(Some)
                .or(just(';').padded().map(|_| None)),
        )
        .map(|((ident, parent), properties)| {
            if let Some(properties) = properties {
                Class::Local {
                    name: ident,
                    parent,
                    properties,
                }
            } else {
                Class::External { name: ident }
            }
        })
}

#[cfg(test)]
mod tests {
    #[test]
    fn external() {
        use super::*;

        assert_eq!(
            class().parse_recovery_verbose("class MyClass;"),
            (
                Some(Class::External {
                    name: crate::Ident("MyClass".to_string())
                }),
                vec![]
            )
        );
    }

    #[test]
    fn local() {
        use super::*;

        assert_eq!(
            class().parse_recovery_verbose("class MyClass { MyProperty = 1; }"),
            (
                Some(Class::Local {
                    name: crate::Ident("MyClass".to_string()),
                    parent: None,
                    properties: vec![crate::Property::Entry {
                        name: crate::Ident("MyProperty".to_string()),
                        value: crate::Value::Number(crate::Number::Int32(1))
                    }]
                }),
                vec![]
            )
        );
    }
}
