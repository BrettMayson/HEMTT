use armake2::run;
use docopt::Docopt;

pub fn run(usage: &[String]) -> Result<(), std::io::Error> {
    let mut args: run::Args = Docopt::new(run::USAGE)
        .and_then(|d| d.argv(usage.iter()).deserialize())
        .unwrap_or_else(|e| e.exit());
    armake2::run::args(&mut args);
    Ok(())
}
