use ariadne::{Color, Fmt, ReportKind};

#[derive(Debug)]
pub struct General {
    message: String,
    ident: Option<String>,
    help: Option<String>,
}

impl General {
    #[must_use]
    pub const fn new(message: String) -> Self {
        Self {
            message,
            ident: None,
            help: None,
        }
    }

    #[must_use]
    pub fn with_ident(mut self, ident: String) -> Self {
        self.ident = Some(ident);
        self
    }

    #[must_use]
    pub fn with_help(mut self, help: String) -> Self {
        self.help = Some(help);
        self
    }

    #[must_use]
    pub fn to_string(&self, kind: ReportKind<'_>) -> String {
        let title = match kind {
            ReportKind::Error => "Error",
            ReportKind::Warning => "Warning",
            ReportKind::Advice => "Advice",
            ReportKind::Custom(w, _) => w,
        };
        let left = format!(
            "[{}]: {}",
            self.ident.as_ref().map_or_else(|| "ERR", |s| s.as_str()),
            title
        )
        .fg(match kind {
            ReportKind::Error => Color::Red,
            ReportKind::Warning => Color::Yellow,
            ReportKind::Advice => Color::Blue,
            ReportKind::Custom(_, c) => c,
        })
        .to_string();
        let top = format!("{} {}", left, self.message);
        match &self.help {
            Some(help) => format!("{}\n{} {}", top, "Help:".fg(Color::Green), help),
            None => top,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error() {
        assert_eq!(
            General::new("This is a test".to_string()).to_string(ReportKind::Error),
            "\u{1b}[31m[ERR]: Error\u{1b}[0m This is a test"
        );
    }

    #[test]
    fn warning() {
        assert_eq!(
            General::new("This is a test".to_string()).to_string(ReportKind::Warning),
            "\u{1b}[33m[ERR]: Warning\u{1b}[0m This is a test"
        );
    }

    #[test]
    fn advice() {
        assert_eq!(
            General::new("This is a test".to_string()).to_string(ReportKind::Advice),
            "\u{1b}[34m[ERR]: Advice\u{1b}[0m This is a test"
        );
    }

    #[test]
    fn custom() {
        assert_eq!(
            General::new("This is a test".to_string())
                .to_string(ReportKind::Custom("Custom", Color::Magenta)),
            "\u{1b}[35m[ERR]: Custom\u{1b}[0m This is a test"
        );
    }

    #[test]
    fn with_ident() {
        assert_eq!(
            General::new("This is a test".to_string())
                .with_ident("TEST".to_string())
                .to_string(ReportKind::Error),
            "\u{1b}[31m[TEST]: Error\u{1b}[0m This is a test"
        );
    }

    #[test]
    fn with_help() {
        assert_eq!(
            General::new("This is a test".to_string())
                .with_help("Help".to_string())
                .to_string(ReportKind::Error),
            "\u{1b}[31m[ERR]: Error\u{1b}[0m This is a test\n\u{1b}[32mHelp:\u{1b}[0m Help"
        );
    }

    #[test]
    fn with_ident_and_help() {
        assert_eq!(
            General::new("This is a test".to_string())
                .with_ident("TEST".to_string())
                .with_help("Help".to_string())
                .to_string(ReportKind::Error),
            "\u{1b}[31m[TEST]: Error\u{1b}[0m This is a test\n\u{1b}[32mHelp:\u{1b}[0m Help"
        );
    }
}
