//! Allows customization of the commands list at runtime in order to facilitate forwards-compatibility.

use std::collections::HashSet;

pub const NULAR_COMMANDS: &[&str] = &include!("./nular_commands.jsonc");
pub const UNARY_COMMANDS: &[&str] = &include!("./unary_commands.jsonc");
pub const BINARY_COMMANDS: &[&str] = &include!("./binary_commands.jsonc");

/// The list of commands that are valid nular command constants for the compiler.
pub const NULAR_COMMANDS_CONSTANTS: &[&str] = &[
    // NOTE: `netobjnull` is not included because it's broken
    // TODO: find out if there are more commands valid as constants
    "nil",
    "confignull",
    "controlnull",
    "diaryrecordnull",
    "displaynull",
    "grpnull",
    "locationnull",
    "objnull",
    "scriptnull",
    "tasknull",
    "teammembernull",
];

/// Nular commands that are alpha-numeric and have special meaning.
///
/// Adding or removing these from a [`Database`] will do nothing, as they are handled manually by the parser.
pub const NULAR_COMMANDS_SPECIAL: &[&str] = &["true", "false"];

/// Binary commands that are alpha-numeric and have special precedence.
///
/// Adding or removing these from a [`Database`] will do nothing, as they are handled manually by the parser.
pub const BINARY_COMMANDS_SPECIAL: &[&str] = &["or", "and", "else", "max", "min", "mod", "atan2"];

/// Commands (operators) that are non-alpha-numeric or have special precedence.
pub const COMMANDS_OPERATORS: &[&str] = &[
    "!", "||", "&&", "==", "!=", ">>", ">=", "<=", ">", "<", "+", "-", "*", "/", "%", "^", ":",
];

/// Contains a list of most nular, unary, and binary commands.
///
/// Instances of [`Database`] should only contain commands that are alpha-numeric, and commands that do not have special precedence.
/// Non-alpha-numeric commands and commands with special precedence are handled manually by the parser.
pub struct Database {
    nular_commands: HashSet<String>,
    unary_commands: HashSet<String>,
    binary_commands: HashSet<String>,
}

impl Database {
    #[must_use]
    /// An empty database with no entries.
    pub fn new() -> Self {
        Self {
            nular_commands: HashSet::new(),
            unary_commands: HashSet::new(),
            binary_commands: HashSet::new(),
        }
    }

    pub fn add_nular_command(&mut self, command: &str) {
        if is_valid_name(command) && !is_in(NULAR_COMMANDS_SPECIAL, command) {
            self.nular_commands.insert(command.to_ascii_lowercase());
        };
    }

    pub fn add_unary_command(&mut self, command: &str) {
        if is_valid_name(command) {
            self.unary_commands.insert(command.to_ascii_lowercase());
        };
    }

    pub fn add_binary_command(&mut self, command: &str) {
        if is_valid_name(command) && !is_in(BINARY_COMMANDS_SPECIAL, command) {
            self.binary_commands.insert(command.to_ascii_lowercase());
        };
    }

    #[must_use]
    pub fn has_nular_command(&self, command: &str) -> bool {
        self.nular_commands.contains(&command.to_ascii_lowercase())
    }

    #[must_use]
    pub fn has_unary_command(&self, command: &str) -> bool {
        self.unary_commands.contains(&command.to_ascii_lowercase())
    }

    #[must_use]
    pub fn has_binary_command(&self, command: &str) -> bool {
        self.binary_commands.contains(&command.to_ascii_lowercase())
    }

    #[must_use]
    pub fn has_command(&self, command: &str) -> bool {
        let command = command.to_ascii_lowercase();
        self.nular_commands.contains(&command)
            || self.unary_commands.contains(&command)
            || self.binary_commands.contains(&command)
    }

    #[must_use]
    pub const fn nular_commands(&self) -> &HashSet<String> {
        &self.nular_commands
    }

    #[must_use]
    pub const fn unary_commands(&self) -> &HashSet<String> {
        &self.unary_commands
    }

    #[must_use]
    pub const fn binary_commands(&self) -> &HashSet<String> {
        &self.binary_commands
    }
}

impl Default for Database {
    fn default() -> Self {
        let mut nular_commands = to_set(NULAR_COMMANDS);
        for &command in NULAR_COMMANDS_SPECIAL {
            nular_commands.remove(command);
        }

        let unary_commands = to_set(UNARY_COMMANDS);

        let mut binary_commands = to_set(BINARY_COMMANDS);
        for &command in BINARY_COMMANDS_SPECIAL {
            binary_commands.remove(command);
        }

        Self {
            nular_commands,
            unary_commands,
            binary_commands,
        }
    }
}

#[must_use]
/// Whether or not this command is valid as a nular constant.
///
/// The given command must be lowercase.
pub fn is_constant_command(command: &str) -> bool {
    is_in(NULAR_COMMANDS_CONSTANTS, command)
}

#[must_use]
/// Whether or not this command is special (has special meaning or precedence).
///
/// The given command must be lowercase.
pub fn is_special_command(command: &str) -> bool {
    is_in(NULAR_COMMANDS_SPECIAL, command) || is_in(BINARY_COMMANDS_SPECIAL, command)
}

#[must_use]
/// Whether or not this command is an operator.
pub fn is_operator_command(command: &str) -> bool {
    is_in(COMMANDS_OPERATORS, command)
}

#[must_use]
/// Whether or not this command is alpha-numeric or a valid command name.
pub fn is_valid_name(command: &str) -> bool {
    command
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

#[must_use]
/// Whether or not this command is alpha-numeric, or an operator command (passes [`is_operator_command`]).
pub fn is_valid_command(command: &str) -> bool {
    is_valid_name(command) || is_operator_command(command)
}

fn to_set(commands: &[&str]) -> HashSet<String> {
    commands
        .iter()
        .map(|command| command.to_lowercase())
        .collect()
}

fn is_in(list: &[&str], item: &str) -> bool {
    list.iter().any(|i| i.eq_ignore_ascii_case(item))
}
