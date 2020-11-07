use super::Node;

#[derive(Clone, Debug)]
pub struct Report {
    pub errors: Vec<Node>,
    pub warnings: Vec<Node>,
}

impl Default for Report {
    fn default() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}
