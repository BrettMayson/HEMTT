use std::sync::Arc;

use hemtt_workspace::reporting::Token;

use crate::{Error, codes::pe17_double_else::DoubleElse};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IfState {
    ReadingIf(Arc<Token>),
    PassingIf(Arc<Token>),
    ReadingElse(Arc<Token>),
    PassingElse(Arc<Token>),
    PassingChild(Arc<Token>),
}

impl IfState {
    pub const fn reading(&self) -> bool {
        match self {
            Self::ReadingIf(_) | Self::ReadingElse(_) => true,
            Self::PassingIf(_) | Self::PassingElse(_) | Self::PassingChild(_) => false,
        }
    }

    pub const fn token(&self) -> &Arc<Token> {
        match self {
            Self::ReadingIf(t)
            | Self::PassingIf(t)
            | Self::ReadingElse(t)
            | Self::PassingElse(t)
            | Self::PassingChild(t) => t,
        }
    }
}

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct IfStates {
    stack: Vec<IfState>,
    did_else: Option<Arc<Token>>,
}

impl IfStates {
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
    pub fn reading(&self) -> bool {
        self.stack.is_empty() || self.stack.iter().all(IfState::reading)
    }

    pub fn push(&mut self, s: IfState) {
        self.did_else = None;
        self.stack.push(s);
    }

    pub fn push_if(&mut self, token: Arc<Token>, state: bool) {
        self.did_else = None;
        if state {
            self.push(IfState::ReadingIf(token));
        } else {
            self.push(IfState::PassingIf(token));
        }
    }

    pub fn pop(&mut self) -> Option<IfState> {
        self.did_else = None;
        self.stack.pop()
    }

    pub fn flip(&mut self, token: Arc<Token>) -> Result<(), Error> {
        if let Some(previous) = self.did_else.take() {
            return Err(DoubleElse::code(
                token.as_ref().clone(),
                previous.as_ref().clone(),
                self.stack
                    .last()
                    .expect("did_else should only be Some if there is a last element in the stack")
                    .token()
                    .as_ref()
                    .clone(),
            ));
        }
        if self
            .stack
            .iter()
            .take(self.stack.len() - 1)
            .all(IfState::reading)
            && let Some(new) = match self.pop() {
                Some(IfState::PassingChild(t)) => Some(IfState::PassingChild(t)),
                Some(IfState::PassingIf(t)) => Some(IfState::ReadingElse(t)),
                Some(IfState::ReadingIf(t)) => Some(IfState::PassingElse(t)),
                Some(IfState::PassingElse(_) | IfState::ReadingElse(_)) | None => None,
            }
        {
            self.push(new);
        }
        self.did_else = Some(token);
        Ok(())
    }
}
