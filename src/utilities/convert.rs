use docopt::Docopt;
use serde::Deserialize;
use serde_arma;
use serde_json;
use serde_transcode;

use std::fs::File;
use std::io::{Write, BufReader, BufWriter};
use std::path::PathBuf;

const USAGE: &'static str = "
Convert Arma 3 Configs to other formats for use in external programs.

Usage:
  convert <input> <output>
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_input: String,
    arg_output: OutputFormat,
}

// Ability to support additonal formats in the future
#[derive(Debug, Deserialize)]
enum OutputFormat {
    JSON,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn run (usage: &Vec<String>) -> Result<(), std::io::Error> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(usage.into_iter()).deserialize())
        .unwrap_or_else(|e| e.exit());

    let reader = BufReader::new(File::open(&args.arg_input).unwrap());

    let mut output = PathBuf::from(args.arg_input);
    output.set_extension(args.arg_output.to_string().to_lowercase());
    let writer = BufWriter::new(File::create(output).unwrap());

    // Only JSON is currently supported
    let mut deserializer = serde_arma::from_reader(reader);
    let mut serializer = serde_json::Serializer::pretty(writer);

    serde_transcode::transcode(&mut deserializer, &mut serializer).unwrap();
    serializer.into_inner().flush().unwrap();

    Ok(())
}
