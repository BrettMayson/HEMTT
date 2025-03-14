use std::path::PathBuf;

use super::{SupportedFile, guess_file_type};
use crate::Error;

#[derive(clap::Args)]
#[allow(clippy::module_name_repetitions)]
pub struct InspectArgs {
    /// file to inspect
    pub(crate) file: String,
}

/// Prints information about an audio file to stdout
///
/// # Errors
/// [`Error::Preprocessor`] if the file can not be preprocessed
pub fn inspect(file: &PathBuf) -> Result<(), Error> {
    let wss = match guess_file_type(file)? {
        Some(SupportedFile::Wss) => {
            let wss = hemtt_wss::Wss::read(std::fs::File::open(file)?)?;
            println!("WSS Details");
            println!("Compression: {:?}", wss.compression());
            println!("Format: {:?}", wss.format());
            println!("Block Align: {:?}", wss.block_align());
            println!("Output Size: {:?}", wss.output_size());
            println!();
            wss
        }
        Some(SupportedFile::Wav) => hemtt_wss::Wss::from_wav(std::fs::File::open(file)?)?,
        Some(SupportedFile::Ogg) => hemtt_wss::Wss::from_ogg(std::fs::File::open(file)?)?,
        Some(SupportedFile::Mp3) => hemtt_wss::Wss::from_mp3(std::fs::File::open(file)?)?,
        _ => {
            println!("Unsupported file type");
            return Ok(());
        }
    };
    println!("Audio Details");
    println!("Channels: {:?}", wss.channels());
    println!("Sample Rate: {:?}", wss.sample_rate());
    println!("Bytes Per Second: {:?}", wss.bytes_per_second());
    println!("Bits Per Sample: {:?}", wss.bits_per_sample());
    Ok(())
}
