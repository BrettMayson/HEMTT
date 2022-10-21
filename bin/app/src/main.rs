use clap::Command;
use hemtt_error::DisplayStyle;

fn cli() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(hemtt_bin_internal::cli().name("internal"))
}

fn main() {
    let matches = cli().get_matches();

    let result = match matches.subcommand() {
        Some(("internal", matches)) => hemtt_bin_internal::execute(matches),
        _ => unreachable!(),
    };
    if let Err(e) = result {
        eprintln!("{}", e.long(DisplayStyle::Error));
        std::process::exit(1);
    }
}
