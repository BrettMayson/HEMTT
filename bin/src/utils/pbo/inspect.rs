use std::fs::File;

use hemtt_pbo::ReadablePbo;
use serde::Serialize;
use tabled::{
    settings::{object::Columns, Alignment, Style},
    Table, Tabled,
};

use crate::{Error, TableFormat};

#[derive(clap::Args)]
pub struct PboInspectArgs {
    /// PBO to inspect
    pub(crate) pbo: String,
    #[clap(long, default_value = "ascii")]
    /// Output format
    pub(crate) format: TableFormat,
}

#[derive(Tabled, Serialize)]
pub struct FileInfo {
    filename: String,
    mime: String,
    size: u32,
    original: u32,
    timestamp: u32,
}

/// Prints information about a [`ReadablePbo`] to stdout
///
/// # Errors
/// [`hemtt_pbo::Error`] if the file is not a valid [`ReadablePbo`]
///
/// # Panics
/// If the file is not a valid [`ReadablePbo`]
pub fn inspect(file: File, format: &TableFormat) -> Result<(), Error> {
    let mut pbo = ReadablePbo::from(file)?;
    println!("Properties");
    for (key, value) in pbo.properties() {
        println!("  - {key}: {value}");
    }
    println!("Checksum (SHA1)");
    let stored = *pbo.checksum();
    println!("  - Stored:  {}", stored.hex());
    let actual = pbo.gen_checksum()?;
    println!("  - Actual:  {}", actual.hex());

    let files = pbo.files();
    println!("Files");
    if pbo.is_sorted().is_ok() {
        println!("  - Sorted: true");
    } else {
        println!("  - Sorted: false !!!");
    }
    println!("  - Count: {}", files.len());
    let data = files
        .iter()
        .map(|file| FileInfo {
            filename: file.filename().to_string(),
            mime: file.mime().to_string(),
            size: file.size(),
            original: file.original(),
            timestamp: file.timestamp(),
        })
        .collect::<Vec<_>>();

    match format {
        TableFormat::Ascii => println!("{}", modify(Table::new(data).with(Style::modern()))),
        TableFormat::Json => println!("{}", serde_json::to_string(&data)?),
        TableFormat::PrettyJson => println!("{}", serde_json::to_string_pretty(&data)?),
        TableFormat::Markdown => println!("{}", modify(Table::new(data).with(Style::markdown()))),
    }

    if pbo.is_sorted().is_err() {
        warn!("The PBO is not sorted, signatures may be invalid");
    }
    if stored != actual {
        warn!("The PBO has an invalid hash stored");
    }
    Ok(())
}

fn modify(table: &mut Table) -> &mut Table {
    table.modify(Columns::new(1..), Alignment::right())
}
