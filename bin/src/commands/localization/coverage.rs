use std::{collections::HashMap, io::BufReader};

use hemtt_stringtable::{Project, Totals};
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
    Table, TableStyle,
};

use crate::{context::Context, report::Report, Error};

macro_rules! missing {
    ($totals:ident, $missing:ident, $lang:tt, $addon:expr) => {
        paste::paste! {
            if $totals.$lang() != $totals.total() {
                $missing.entry(stringify!($lang)).or_insert_with(Vec::new).push($addon);
            }
        }
    };
}

fn first_capital(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or_else(String::new, |f| {
        f.to_uppercase().collect::<String>() + c.as_str()
    })
}

macro_rules! row {
    ($table:ident, $global:ident, $missing:ident, $lang:tt) => {
        if $global.$lang() != 0 {
            $table.add_row(Row::new(vec![
                TableCell::new(first_capital(stringify!($lang))),
                TableCell::new(format!(
                    "{:.2}%",
                    f64::from($global.$lang()) / f64::from($global.total()) * 100.0
                )),
                TableCell::new($global.$lang()),
                {
                    if let Some(missing) = $missing.get(stringify!($lang)) {
                        if missing.len() < 3 {
                            TableCell::new(missing.join(", "))
                        } else {
                            TableCell::new(format!("{} and {} more", missing[0], missing.len() - 1))
                        }
                    } else {
                        TableCell::new("")
                    }
                },
            ]));
        }
    };
}

pub fn coverage() -> Result<Report, Error> {
    let ctx = Context::new(None, crate::context::PreservePrevious::Remove, true)?;

    let mut global = Totals::default();
    let mut missing = HashMap::new();

    for addon in ctx.addons() {
        let stringtable_path = ctx
            .workspace_path()
            .join(addon.folder())?
            .join("stringtable.xml")?;
        if stringtable_path.exists()? {
            let project = match Project::from_reader(BufReader::new(stringtable_path.open_file()?))
            {
                Ok(project) => project,
                Err(e) => {
                    error!("Failed to read stringtable for {}", addon.folder());
                    error!("{:?}", e);
                    return Ok(Report::new());
                }
            };
            project.packages().iter().for_each(|package| {
                let totals = package.totals();
                missing!(totals, missing, original, addon.folder());
                missing!(totals, missing, english, addon.folder());
                missing!(totals, missing, czech, addon.folder());
                missing!(totals, missing, french, addon.folder());
                missing!(totals, missing, spanish, addon.folder());
                missing!(totals, missing, italian, addon.folder());
                missing!(totals, missing, polish, addon.folder());
                missing!(totals, missing, portuguese, addon.folder());
                missing!(totals, missing, russian, addon.folder());
                missing!(totals, missing, german, addon.folder());
                missing!(totals, missing, korean, addon.folder());
                missing!(totals, missing, japanese, addon.folder());
                missing!(totals, missing, chinese, addon.folder());
                missing!(totals, missing, chinesesimp, addon.folder());
                missing!(totals, missing, turkish, addon.folder());
                missing!(totals, missing, swedish, addon.folder());
                missing!(totals, missing, slovak, addon.folder());
                missing!(totals, missing, serbocroatian, addon.folder());
                missing!(totals, missing, norwegian, addon.folder());
                missing!(totals, missing, icelandic, addon.folder());
                missing!(totals, missing, hungarian, addon.folder());
                missing!(totals, missing, greek, addon.folder());
                missing!(totals, missing, finnish, addon.folder());
                missing!(totals, missing, dutch, addon.folder());
                global.merge(&totals);
            });
        }
    }

    let mut table = Table::new();
    table.style = TableStyle::thin();
    table.add_row(Row::new(vec![
        TableCell::builder("Language")
            .alignment(Alignment::Center)
            .build(),
        TableCell::builder("Percent")
            .alignment(Alignment::Center)
            .build(),
        TableCell::builder("Total")
            .alignment(Alignment::Center)
            .build(),
        TableCell::builder("Missing Addons")
            .alignment(Alignment::Center)
            .build(),
    ]));

    row!(table, global, missing, original);
    row!(table, global, missing, english);
    row!(table, global, missing, czech);
    row!(table, global, missing, french);
    row!(table, global, missing, spanish);
    row!(table, global, missing, italian);
    row!(table, global, missing, polish);
    row!(table, global, missing, portuguese);
    row!(table, global, missing, russian);
    row!(table, global, missing, german);
    row!(table, global, missing, korean);
    row!(table, global, missing, japanese);
    row!(table, global, missing, chinese);
    row!(table, global, missing, chinesesimp);
    row!(table, global, missing, turkish);
    row!(table, global, missing, swedish);
    row!(table, global, missing, slovak);
    row!(table, global, missing, serbocroatian);
    row!(table, global, missing, norwegian);
    row!(table, global, missing, icelandic);
    row!(table, global, missing, hungarian);
    row!(table, global, missing, greek);
    row!(table, global, missing, finnish);
    row!(table, global, missing, dutch);

    println!("{}", table.render());

    Ok(Report::new())
}
