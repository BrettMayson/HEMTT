mod case;

use crate::Error;

#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
/// Tools for working with SQF (Status Quo Function) script files.
pub struct Command {
    #[command(subcommand)]
    commands: Subcommands,
}

#[derive(clap::Subcommand)]
enum Subcommands {
    #[command(verbatim_doc_comment)]
    /// This will recursively correct all capitalization mistakes in SQF commands.
    ///
    /// ```admonish danger
    /// This command requires **manual review**. It can have lots of false positives so you are **strongly encouraged** to check each modified file to ensure it is correct.
    /// ```
    ///
    /// ## Example
    ///
    /// ```sqf
    /// private _positionASL = GetPosasl Player;
    /// // becomes
    /// private _positionASL = getPosASL player;
    /// ```
    ///
    /// ## False Positives
    ///
    /// This command does not full parse your SQF files.
    ///
    /// It will not change words in strings in comments, but it may change words that will break your program
    ///
    /// ```sqf
    /// // script_macros.hpp
    /// #define FALSE 0
    /// #define TRUE 1
    ///
    /// // fnc_someFunction.sqf
    /// if (getNumber (configFile >> "someClass" >= TRUE)) then {...};
    /// // becomes
    /// if (getNumber (configFile >> "someClass" >= true)) then {...};
    /// ```
    ///
    /// ```sqf
    /// private _value = player getVariable [QGVAR(showHud), false];
    /// // becomes
    /// private _value = player getVariable [QGVAR(showHUD), false];
    /// ```
    Case(case::SqfCaseArgs),
}

/// Execute the paa command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If the args are not present from clap
pub fn execute(cmd: &Command) -> Result<(), Error> {
    match &cmd.commands {
        Subcommands::Case(args) => case::execute(args),
    }
}
