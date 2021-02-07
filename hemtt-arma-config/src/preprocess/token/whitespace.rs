#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "maps", derive(serde::Serialize))]
pub enum Whitespace {
    Space,
    Tab,
}

impl ToString for Whitespace {
    fn to_string(&self) -> String {
        match self {
            Whitespace::Space => " ",
            Whitespace::Tab => "\t",
        }
        .to_string()
    }
}
