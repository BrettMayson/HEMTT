use std::path::PathBuf;

use crate::Error;

#[derive(clap::Args)]
/// Convert images to and from PAA format
pub struct PaaConvertArgs {
    /// Source file (PAA or image)
    src: String,
    /// Destination file (PAA or image)
    dest: String,
}

fn invalid_data(message: impl Into<String>) -> Error {
    Error::Io(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        message.into(),
    ))
}

/// Execute the convert command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(args: &PaaConvertArgs) -> Result<(), Error> {
    let from = PathBuf::from(&args.src);
    let output = PathBuf::from(&args.dest);
    if output.exists() {
        error!("Output file already exists");
        return Ok(());
    }
    if ["paa", "pac"].contains(
        &from
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_lowercase()
            .as_str(),
    ) {
        let paa = hemtt_paa::Paa::read(fs_err::File::open(from)?)?;
        let Some(map) = paa.maps().first() else {
            return Err(invalid_data("PAA has no maps"));
        };
        let image = map.0.get_image().map_err(invalid_data)?;
        image.save(output)?;
        info!("PAA converted");
    } else {
        let image = image::open(from)?;
        let format = hemtt_paa::PaXType::for_dynamic(&image);
        let continue_conversion = confirm_format(format, &args.src)?;
        if !continue_conversion {
            return Ok(());
        }
        let paa = hemtt_paa::Paa::from_dynamic(&image, format)?;
        let mut file = fs_err::File::create(output)?;
        paa.write(&mut file)?;
        info!("Image converted to {} PAA", format.to_string());
    }
    Ok(())
}

/// Confirm whether to use the selected format.
///
/// # Errors
/// Returns an error if the confirmation prompt fails.
pub fn confirm_format(format: hemtt_paa::PaXType, path: &str) -> Result<bool, Error> {
    if format == hemtt_paa::PaXType::ARGB8 {
        warn!("Using ARGB8 format for non-power-of-two image: {}", path);
        warn!("This is likely unintended and will not work as a standard texture.");
        if !dialoguer::Confirm::new()
            .with_prompt("Continue?")
            .default(false)
            .interact()?
        {
            return Ok(false);
        }
    }
    Ok(true)
}
