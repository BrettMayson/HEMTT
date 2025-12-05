use std::path::PathBuf;

use hemtt_wss::Compression;
use tabled::Tabled;

use crate::{Error, utils::bytes_to_human_readable};

#[derive(clap::Args)]
#[allow(clippy::module_name_repetitions)]
pub struct CompressArgs {}

/// Checks for wss files that can be compressed
///
/// # Errors
/// [`Error::Wss`] if their are issues with audio files
#[allow(clippy::cast_precision_loss)]
pub fn compress() -> Result<(), Error> {
    let entries = walkdir::WalkDir::new(".")
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().is_file() && entry.path().extension().unwrap_or_default() == "wss"
            {
                Some(entry)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let mut candidates = Vec::new();
    let mut current = 0;
    let mut possible = 0;
    for entry in entries {
        let Ok(wss) = hemtt_wss::Wss::read(fs_err::File::open(entry.path())?) else {
            println!("Failed to read wss file: {}", entry.path().display());
            continue;
        };
        if wss.compression() != &Compression::Nibble {
            let raw_size = wss.size();
            let mut size = raw_size;
            if wss.compression() == &Compression::Byte {
                size /= 2;
            }
            current += size;
            possible += raw_size / 4;
            candidates.push(Conversion {
                path: entry.path().display().to_string(),
                compression: wss.compression().as_str(),
                current: bytes_to_human_readable(size as u64),
                compressed: bytes_to_human_readable(raw_size as u64 / 4),
            });
        }
    }
    if candidates.is_empty() {
        println!("No candidates found");
        return Ok(());
    }

    println!(
        "{}",
        modify(tabled::Table::new(&candidates).with(tabled::settings::Style::modern()))
    );

    println!("Found {} candidates", candidates.len());
    println!(
        "Current size: {}, possible size: {}, saved {}%",
        bytes_to_human_readable(current as u64),
        bytes_to_human_readable(possible as u64),
        (current as f64 / possible as f64)
            .mul_add(-100.0, 100.0)
            .round()
            .abs()
    );

    if !dialoguer::Confirm::new()
        .with_prompt("Do you want to continue? (Make sure to backup your files!)")
        .interact()
        .unwrap_or_default()
    {
        return Ok(());
    }

    for entry in candidates {
        let path = PathBuf::from(entry.path);
        let mut wss = hemtt_wss::Wss::read(fs_err::File::open(&path)?)?;
        let mut buffer = Vec::new();
        wss.set_compression(Compression::Nibble);
        wss.write(&mut buffer)?;
        fs_err::write(path, buffer)?;
    }

    Ok(())
}

#[derive(Tabled)]
pub struct Conversion {
    path: String,
    compression: &'static str,
    current: String,
    compressed: String,
}

fn modify(table: &mut tabled::Table) -> &mut tabled::Table {
    table.modify(
        tabled::settings::object::Columns::new(1..),
        tabled::settings::Alignment::right(),
    )
}
