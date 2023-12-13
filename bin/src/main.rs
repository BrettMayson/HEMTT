use tracing::error;

fn main() {
    std::panic::set_hook(Box::new(|panic| {
        error!("{panic}");
        eprintln!(
            r#"
Oh no! HEMTT has crashed!
This is a bug in HEMTT itself, not necessarily your project.
Even if there is a bug in your project, HEMTT should not crash, but gracefully exit with an error message.

Support for HEMTT can be found on:
GitHub (https://github.com/BrettMayson/HEMTT)
#hemtt on the ACE 3 Discord (https://acemod.org/discord)

The log from the most recent run can be found in `.hemttout/latest.log`.

It is always best to the include the log and a link to your project when reporting a bug, this will help reproduce the issue.
"#
        );
        std::process::exit(1);
    }));

    #[cfg(windows)]
    if enable_ansi_support::enable_ansi_support().is_err() {
        tracing::warn!("Failed to enable ANSI support, colored output will not work");
    }

    if let Err(e) = hemtt::execute(&hemtt::cli().get_matches()) {
        if let hemtt::Error::Preprocessor(e) = &e {
            if let Some(code) = e.get_code() {
                if let Some(report) = code.report() {
                    eprintln!("{report}");
                    std::process::exit(1);
                }
            }
        }
        error!("{}", e);
        std::process::exit(1);
    }
}
