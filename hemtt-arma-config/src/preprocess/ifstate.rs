#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum IfState {
    ReadingIf,
    PassingIf,
    ReadingElse,
    PassingElse,
}

impl IfState {
    pub fn reading(&self) -> bool {
        match &self {
            IfState::ReadingIf => true,
            IfState::PassingIf => false,
            IfState::ReadingElse => true,
            IfState::PassingElse => false,
        }
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct IfStates(Vec<IfState>);
impl IfStates {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn reading(&self) -> bool {
        if let Some(l) = self.0.last() {
            l.reading()
        } else {
            true
        }
    }

    pub fn push(&mut self, s: IfState) {
        println!("Pushing if state: {:?}", s);
        self.0.push(s)
    }

    pub fn pop(&mut self) -> Option<IfState> {
        self.0.pop()
    }
}
