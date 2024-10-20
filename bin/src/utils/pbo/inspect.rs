use std::fs::File;

use hemtt_pbo::ReadablePbo;
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
    Table, TableStyle,
};

use crate::Error;

/// Prints information about a [`ReadablePbo`] to stdout
///
/// # Errors
/// [`hemtt_pbo::Error`] if the file is not a valid [`ReadablePbo`]
///
/// # Panics
/// If the file is not a valid [`ReadablePbo`]
pub fn inspect(file: File) -> Result<(), Error> {
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
