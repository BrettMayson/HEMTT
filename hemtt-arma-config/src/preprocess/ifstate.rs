#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum IfState {
    ReadingIf,
    PassingIf,
    ReadingElse,
    PassingElse,
    PassingChild,
}

impl IfState {
    pub fn reading(&self) -> bool {
        match &self {
            IfState::ReadingIf => true,
            IfState::PassingIf => false,
            IfState::ReadingElse => true,
            IfState::PassingElse => false,
            IfState::PassingChild => false,
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
        self.0.iter().all(|f| f.reading())
    }

    pub fn push(&mut self, s: IfState) {
        self.0.push(s)
    }

    pub fn pop(&mut self) -> Option<IfState> {
        self.0.pop()
    }

    pub fn flip(&mut self) {
        if self.0.iter().take(self.0.len() - 1).all(|f| f.reading()) {
            if let Some(new) = match self.pop() {
                Some(IfState::PassingChild) => Some(IfState::PassingChild),
                Some(IfState::PassingIf) => Some(IfState::ReadingElse),
                Some(IfState::ReadingIf) => Some(IfState::PassingElse),
                Some(IfState::PassingElse) => None,
                Some(IfState::ReadingElse) => None,
                None => None,
            } {
                self.push(new);
            }
        }
    }
}
