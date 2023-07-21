use crate::Property;

#[derive(Debug, PartialEq)]
/// A config file
pub struct Config(pub Vec<Property>);
