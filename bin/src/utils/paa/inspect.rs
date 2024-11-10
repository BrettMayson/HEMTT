use std::fs::File;

use serde::Serialize;
use tabled::{
    settings::{Alignment, Style},
    Table, Tabled,
};

use crate::{Error, TableFormat};

#[derive(clap::Args)]
pub struct PaaInspectArgs {
    /// PAA to inspect
    pub(crate) paa: String,
    #[clap(long, default_value = "ascii")]
    /// Output format
    pub(crate) format: TableFormat,
}

#[derive(Tabled, Serialize)]
pub struct MipMapInfo {
    #[tabled(rename = "Width")]
    width: u16,
    #[tabled(rename = "Height")]
    height: u16,
    #[tabled(rename = "Size")]
    size: usize,
    #[tabled(rename = "Format")]
    format: String,
    #[tabled(rename = "Compressed")]
    compressed: bool,
}

/// Prints information about a PAA to stdout
///
/// # Errors
/// [`Error::Io`] if the file is not a valid [`hemtt_paa::Paa`]
pub fn inspect(mut file: File, format: &TableFormat) -> Result<(), Error> {
    let paa = hemtt_paa::Paa::read(&mut file)?;
    println!("PAA");
    println!("  - Format: {}", paa.format());
    let maps = paa.maps();
    println!("Maps: {}", maps.len());
    let data = maps
        .iter()
        .map(|map| MipMapInfo {
            width: map.width(),
            height: map.height(),
            size: map.data().len(),
            format: format!("{:?}", map.format()),
            compressed: map.is_compressed(),
        })
        .collect::<Vec<_>>();

    match format {
        TableFormat::Ascii => println!(
            "{}",
            Table::new(data)
                .with(Style::modern())
                .with(Alignment::right())
        ),
        TableFormat::Json => println!("{}", serde_json::to_string_pretty(&data)?),
        TableFormat::Markdown => println!(
            "{}",
            Table::new(data)
                .with(Style::markdown())
                .with(Alignment::right())
        ),
    }

    Ok(())
}
