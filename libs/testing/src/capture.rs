use std::io::Read;

/// Captures everything written to `stdout` during its lifetime.
///
/// Example:
/// ```
/// let capture = StdoutCapture::new();
/// println!("hello world");
/// let output = capture.finish();
/// assert!(output.contains("hello world"));
/// ```
pub struct OutputCapture {
    reader: std::io::Chain<gag::BufferRedirect, gag::BufferRedirect>,
}

impl OutputCapture {
    #[must_use]
    /// Start capturing stdout and stderr
    ///
    /// # Panics
    /// - Panics if stdout or stderr cannot be redirected
    pub fn new() -> Self {
        let buf = gag::BufferRedirect::stdout()
            .expect("failed to redirect stdout")
            .chain(gag::BufferRedirect::stderr().expect("failed to redirect stderr"));
        Self { reader: buf }
    }

    #[must_use]
    /// Stop capturing and return all captured output as a `String`
    pub fn finish(mut self) -> String {
        let mut output = String::new();
        let _ = self.reader.read_to_string(&mut output);
        output.replace('\r', "")
    }
}

impl Default for OutputCapture {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::OutputCapture;

    #[test]
    fn test_stdout_capture() {
        let capture = OutputCapture::new();
        println!("Hello, world!");
        let output = capture.finish();
        assert!(output.contains("Hello, world!"));
    }
}
