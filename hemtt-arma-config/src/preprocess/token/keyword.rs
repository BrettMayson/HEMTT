#[derive(Clone, Debug, PartialEq)]
pub enum Keyword {
    Class,
    Delete,
    Enum,
}

impl ToString for Keyword {
    fn to_string(&self) -> String {
        match self {
            Keyword::Class => "class",
            Keyword::Delete => "delete",
            Keyword::Enum => "enum",
        }
        .to_string()
    }
}
