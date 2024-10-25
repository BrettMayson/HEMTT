use chumsky::error::Simple;
use hemtt_workspace::reporting::{Code, Diagnostic, Processed};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
/// A chumsky error
pub struct ChumskyCode {
    err: Simple<char>,
    diagnostic: Option<Diagnostic>,
}

impl Code for ChumskyCode {
    fn ident(&self) -> &'static str {
        "CCHU"
    }

    fn message(&self) -> String {
        self.err.to_string()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl ChumskyCode {
    #[must_use]
    pub fn new(err: Simple<char>, processed: &Processed) -> Self {
        Self {
            err,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.err.span(), processed);
        if let Some(diag) = &mut self.diagnostic {
            diag.notes.push(format!(
                "The processed output of the line with the error was:\n{} ",
                {
                    let mut start =
                        std::cmp::min(self.err.span().start, processed.as_str().len() - 1);
                    while start > 0 && processed.as_str().as_bytes()[start] != b'\n' {
                        start -= 1;
                    }
                    while start < self.err.span().start
                        && processed.as_str().as_bytes()[start].is_ascii_whitespace()
                    {
                        start += 1;
                    }
                    let mut end = self.err.span().end;
                    while end < processed.as_str().len()
                        && processed.as_str().as_bytes()[end] != b'\n'
                    {
                        end += 1;
                    }
                    processed.extract(start..end).to_string()
                }
            ));
        }
        self
    }
}
