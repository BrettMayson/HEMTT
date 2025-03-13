use std::path::PathBuf;

use hemtt_wss::Compression;

use crate::Error;

use super::{SupportedFile, guess_file_type};

#[derive(clap::Args)]
#[allow(clippy::module_name_repetitions)]
pub struct ConvertArgs {
    /// file to convert
    pub(crate) file: String,
    /// output file or extension
    pub(crate) output: String,
    /// compression (wss only)
    #[arg(long, short)]
    pub(crate) compression: Option<u8>,
}

pub fn convert(file: &PathBuf, output: &str, compression: Option<u8>) -> Result<(), Error> {
    let compression = Compression::from_u32(u32::from(compression.unwrap_or(0)))?;
    let wss = match guess_file_type(file)? {
        Some(SupportedFile::Wss) => {
            let mut wss = hemtt_wss::Wss::read(std::fs::File::open(file)?)?;
            wss.set_compression(compression);
            wss
        }
        Some(SupportedFile::Wav) => {
            hemtt_wss::Wss::from_wav_with_compression(std::fs::File::open(file)?, compression)?
        }
        Some(SupportedFile::Ogg) => {
            hemtt_wss::Wss::from_ogg_with_compression(std::fs::File::open(file)?, compression)?
        }
        Some(SupportedFile::Mp3) => {
            hemtt_wss::Wss::from_mp3_with_compression(std::fs::File::open(file)?, compression)?
        }
        _ => {
            println!("Unsupported file type");
            return Ok(());
        }
    };
    let (extension, output) = if output.contains('.') {
        let output = PathBuf::from(output);
        (
            output
                .extension()
                .expect("No extension")
                .to_str()
                .expect("extension is not valid")
                .to_string(),
            output,
        )
    } else {
        let output_file = file.with_extension(output);
        (output.to_string(), output_file)
    };

    let data = match extension.as_str() {
        "wss" => {
            let mut buffer = Vec::new();
            wss.write(&mut buffer)?;
            buffer
        }
        "wav" => wss.to_wav()?,
        "ogg" => wss.to_ogg()?,
        _ => {
            println!("Unsupported file type to convert to: {extension}");
            return Ok(());
        }
    };
    std::io::Write::write_all(&mut std::fs::File::create(&output)?, &data)?;
    println!("Converted to: {}", output.display());
    Ok(())
}
