use super::{message::Message, Report};

#[derive(Debug, Default)]
pub struct Output<T> {
    result: Option<T>,
    warnings: Vec<Message>,
    errors: Vec<Message>,
}

impl<T> Output<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            result: None,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn warn<M: Into<Message>>(&mut self, warning: M) {
        self.warnings.push(warning.into());
    }

    pub fn error<M: Into<Message>>(&mut self, error: M) {
        self.errors.push(error.into());
    }

    pub fn result(self, report: &mut Report) -> Option<T> {
        report.add_warnings(self.warnings);
        report.add_errors(self.errors);
        self.result
    }

    pub fn warnings(&self) -> &[Message] {
        &self.warnings
    }

    pub fn errors(&self) -> &[Message] {
        &self.errors
    }

    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}
