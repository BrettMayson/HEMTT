use std::{
    fs::File,
    io::{Read, Seek},
    path::PathBuf,
};

use clap::{ArgMatches, Command};
use hemtt_pbo::ReadablePbo;
use hemtt_signing::{BIPublicKey, BISign};
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
    Table, TableStyle,
};

use crate::Error;

#[must_use]
pub fn cli() -> Command {
    Command::new("inspect")
        .about("Inspect an Arma file")
        .long_about("Provides information about supported files. Supported: pbo, bikey, bisign")
        .arg(
            clap::Arg::new("file")
                .help("File to inspect")
                .required(true),
        )
}

/// Execute the inspect command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If the args are not present from clap
pub fn execute(matches: &ArgMatches) -> Result<(), Error> {
    let path = PathBuf::from(matches.get_one::<String>("file").expect("required"));
    match path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
    {
        "paa" => {
            paa(File::open(&path)?)?;
        }
        "pbo" => {
            pbo(File::open(&path)?)?;
        }
        "bikey" => {
            bikey(File::open(&path)?, &path)?;
        }
        "bisign" => {
            bisign(File::open(&path)?, &path)?;
        }
        _ => {
            let mut file = File::open(&path)?;
            let buf = &mut [0u8; 6];
            file.read_exact(buf)?;
            file.seek(std::io::SeekFrom::Start(0))?;
            // PBO
            if buf == b"\x00sreV\x00" {
                warn!("The file appears to be a PBO but does not have the .pbo extension.");
                pbo(file)?;
                return Ok(());
            }
            // PAA (skip first two bytes)
            if &buf[2..] == b"GGAT" {
                warn!("The file appears to be a PAA but does not have the .paa extension.");
                file.seek(std::io::SeekFrom::Start(0))?;
                paa(file)?;
                return Ok(());
            }
            // BiSign
            if BISign::read(&mut file).is_ok() {
                warn!("The file appears to be a BiSign but does not have the .bisign extension.");
                file.seek(std::io::SeekFrom::Start(0))?;
                bisign(file, &path)?;
                return Ok(());
            }
            file.seek(std::io::SeekFrom::Start(0))?;
            // BiPublicKey
            if BIPublicKey::read(&mut file).is_ok() {
                warn!(
                    "The file appears to be a BiPublicKey but does not have the .bikey extension."
                );
                file.seek(std::io::SeekFrom::Start(0))?;
                bikey(file, &path)?;
                return Ok(());
            }
            println!("Unsupported file type");
        }
    }
    Ok(())
}

/// Prints information about a [`BIPublicKey`] to stdout
///
/// # Errors
/// [`hemtt_signing::Error`] if the file is not a valid [`BIPublicKey`]
pub fn bikey(mut file: File, path: &PathBuf) -> Result<BIPublicKey, Error> {
    let publickey = BIPublicKey::read(&mut file)?;
    println!("Public Key: {path:?}");
    println!("  - Authority: {}", publickey.authority());
    println!("  - Length: {}", publickey.length());
    println!("  - Exponent: {}", publickey.exponent());
    println!("  - Modulus: {}", publickey.modulus_display(13));
    Ok(publickey)
}

/// Prints information about a [`BISign`] to stdout
///
/// # Errors
/// [`hemtt_signing::Error`] if the file is not a valid [`BISign`]
pub fn bisign(mut file: File, path: &PathBuf) -> Result<BISign, Error> {
    let signature = BISign::read(&mut file)?;
    println!("Signature: {path:?}");
    println!("  - Authority: {}", signature.authority());
    println!("  - Version: {}", signature.version());
    println!("  - Length: {}", signature.length());
    println!("  - Exponent: {}", signature.exponent());
    println!("  - Modulus: {}", signature.modulus_display(13));
    Ok(signature)
}

/// Prints information about a [`ReadablePbo`] to stdout
///
/// # Errors
/// [`hemtt_pbo::Error`] if the file is not a valid [`ReadablePbo`]
///
/// # Panics
/// If the file is not a valid [`ReadablePbo`]
pub fn pbo(file: File) -> Result<(), Error> {
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
    let mut table = Table::new();
    table.style = TableStyle::thin();
    table.add_row(Row::new(vec![
        TableCell::builder("Filename")
            .alignment(Alignment::Center)
            .build(),
        TableCell::builder("Method")
            .alignment(Alignment::Center)
            .build(),
        TableCell::builder("Size")
            .alignment(Alignment::Center)
            .build(),
        TableCell::builder("Original")
            .alignment(Alignment::Center)
            .build(),
        TableCell::builder("Timestamp")
            .alignment(Alignment::Center)
            .build(),
    ]));
    for file in files {
        let mut row = Row::new(vec![
            TableCell::new(file.filename()),
            TableCell::builder(file.mime().to_string())
                .alignment(Alignment::Right)
                .build(),
            TableCell::builder(file.size().to_string())
                .alignment(Alignment::Right)
                .build(),
            TableCell::builder(file.original().to_string())
                .alignment(Alignment::Right)
                .build(),
            TableCell::builder(file.timestamp().to_string())
                .alignment(Alignment::Right)
                .build(),
        ]);
        row.has_separator = table.rows.len() == 1;
        table.add_row(row);
    }
    println!("{}", table.render());
    if pbo.is_sorted().is_err() {
        warn!("The PBO is not sorted, signatures may be invalid");
    }
    if stored != actual {
        warn!("The PBO has an invalid hash stored");
    }
    Ok(())
}

/// Prints information about a PAA to stdout
///
/// # Errors
/// [`Error::Io`] if the file is not a valid [`hemtt_paa::Paa`]
pub fn paa(mut file: File) -> Result<(), Error> {
    let paa = hemtt_paa::Paa::read(&mut file)?;
    println!("PAA");
    println!("  - Format: {}", paa.format());
    let maps = paa.maps();
    println!("Maps: {}", maps.len());
    let mut table = Table::new();
    table.style = TableStyle::thin();
    table.add_row(Row::new(vec![
        TableCell::builder("Width")
            .alignment(Alignment::Center)
            .build(),
        TableCell::builder("Height")
            .alignment(Alignment::Center)
            .build(),
        TableCell::builder("Size")
            .alignment(Alignment::Center)
            .build(),
        TableCell::builder("Format")
            .alignment(Alignment::Center)
            .build(),
        TableCell::builder("Compressed")
            .alignment(Alignment::Center)
            .build(),
    ]));
    for map in maps {
        let mut row = Row::new(vec![
            TableCell::builder(map.width().to_string())
                .alignment(Alignment::Right)
                .build(),
            TableCell::builder(map.height().to_string())
                .alignment(Alignment::Right)
                .build(),
            TableCell::builder(map.data().len().to_string())
                .alignment(Alignment::Right)
                .build(),
            TableCell::builder(format!("{:?}", map.format()))
                .alignment(Alignment::Right)
                .build(),
            TableCell::builder(map.is_compressed().to_string())
                .alignment(Alignment::Right)
                .build(),
        ]);
        row.has_separator = table.rows.len() == 1;
        table.add_row(row);
    }
    println!("{}", table.render());
    Ok(())
}
