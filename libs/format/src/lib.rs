mod config;
mod sqf;

pub use config::format_config;
pub use sqf::format_sqf;

pub const CONFIG_EXTENSIONS: [&str; 5] = ["hpp", "cpp", "rvmat", "ext", "inc"];
pub const SQF_EXTENSIONS: [&str; 2] = ["sqf", "fsm"];

#[derive(Debug)]
pub enum IndentStyle {
    Space,
    Tab,
}

#[derive(Debug)]
pub struct FormatterConfig {
    indent_style: IndentStyle,
    indent_size: usize,
    space_before_brace: bool,
}

impl FormatterConfig {
    fn indent(&self, level: usize) -> String {
        match self.indent_style {
            IndentStyle::Space => " ".repeat(self.indent_size * level),
            IndentStyle::Tab => "\t".repeat(level),
        }
    }
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            indent_style: IndentStyle::Space,
            indent_size: 4,
            space_before_brace: true,
        }
    }
}
