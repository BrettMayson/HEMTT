use std::fs::File;

use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
    Table, TableStyle,
};

use crate::Error;

/// Prints information about a PAA to stdout
///
/// # Errors
/// [`Error::Io`] if the file is not a valid [`hemtt_paa::Paa`]
pub fn inspect(mut file: File) -> Result<(), Error> {
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
