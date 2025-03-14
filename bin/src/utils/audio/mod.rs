use std::{
    io::{Read, Seek},
    path::PathBuf,
};

use crate::Error;

mod compress;
mod convert;
mod inspect;

pub use inspect::inspect;

#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
/// Commands for audio files
pub struct Command {
    #[command(subcommand)]
    commands: Subcommands,
}

#[derive(clap::Subcommand)]
enum Subcommands {
    /// Inspect an audio file
    Inspect(inspect::InspectArgs),
    /// Convert an audio file
    Convert(convert::ConvertArgs),
    /// Compress wss files
    Compress(compress::CompressArgs),
}

/// Execute the audio command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If the args are not present from clap
pub fn execute(cmd: &Command) -> Result<(), Error> {
    match &cmd.commands {
        Subcommands::Inspect(args) => inspect::inspect(&PathBuf::from(&args.file)),
        Subcommands::Convert(args) => {
            convert::convert(&PathBuf::from(&args.file), &args.output, args.compression)
        }
        Subcommands::Compress(_) => compress::compress(),
    }
}

pub enum SupportedFile {
    Mp3,
    Wav,
    Ogg,
    Wss,
}

/// Guess the file type of an audio file based on the first few bytes
///
/// # Errors
/// [`std::io::Error`] if an IO error occurs
pub fn guess_file_type(file: &PathBuf) -> Result<Option<SupportedFile>, Error> {
    let mut file = std::fs::File::open(file)?;
    let buf = &mut [0u8; 12];
    file.read_exact(buf)?;
    file.seek(std::io::SeekFrom::Start(0))?;
    // WSS
    if &buf[0..4] == b"WSS0" {
        return Ok(Some(SupportedFile::Wss));
    }
    // WAV
    if &buf[0..4] == b"RIFF" && &buf[8..12] == b"WAVE" {
        return Ok(Some(SupportedFile::Wav));
    }
    // OGG
    if &buf[0..4] == b"OggS" {
        return Ok(Some(SupportedFile::Ogg));
    }
    // MP3
    if &buf[0..3] == b"ID3" {
        return Ok(Some(SupportedFile::Mp3));
    }
    Ok(None)
}
