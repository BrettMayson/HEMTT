use crate::{Stage, Task};

#[derive(Clone)]
pub struct Step {
    pub tasks: Vec<Box<dyn Task>>,
    pub name: String,
    pub none: bool,
    pub parallel: bool,
    pub stage: Stage,
}
impl Step {
    pub fn parallel(name: &str, stage: Stage, tasks: Vec<Box<dyn Task>>) -> Self {
        Self {
            name: name.to_string(),
            stage,
            tasks,
            none: false,
            parallel: true,
        }
    }

    pub fn single(name: &str, stage: Stage, tasks: Vec<Box<dyn Task>>) -> Self {
        Self {
            name: name.to_string(),
            stage,
            tasks,
            none: false,
            parallel: false,
        }
    }

    pub fn none() -> Self {
        Self {
            name: "".to_string(),
            stage: Stage::None,
            tasks: Vec::new(),
            none: true,
            parallel: false,
        }
    }
}
