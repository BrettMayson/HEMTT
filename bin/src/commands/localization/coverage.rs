use std::collections::HashMap;

use hemtt_stringtable::{Project, Totals};
use serde::Serialize;
use tabled::{Table, Tabled, settings::Style};

use crate::{Error, TableFormat, context::Context, report::Report};

#[derive(clap::Parser)]
#[allow(clippy::module_name_repetitions)]
/// Generate a coverage report
///
/// HEMTT will display a table of the coverage of
/// language localization in the project. Showing the
/// percentage, total strings, and how many
/// addons have gaps in their localization.
///
/// ## Report Details
///
/// For each supported language, the report shows:
/// - Coverage percentage (how many strings are translated)
/// - List of addons with missing translations
///
/// This helps identify which addons need localization work and tracks
/// translation progress across your project.
pub struct Command {
    #[arg(long, default_value = "ascii")]
    /// Output format
    format: TableFormat,
}

macro_rules! missing {
    ($totals:ident, $missing:ident, $lang:tt, $addon:expr) => {
        paste::paste! {
            if $totals.$lang() != $totals.total() {
                $missing.entry(stringify!($lang)).or_insert_with(Vec::new).push($addon);
            }
        }
    };
}

macro_rules! row {
    ($table:ident, $global:ident, $missing:ident, $lang:ident) => {
        paste::paste! {
            if $global.$lang() != 0 {
                $table.push(Entry {
                    language: first_capital(stringify!($lang)),
                    percent: Percentage(f64::from($global.$lang()) / f64::from($global.total()) * 100.0),
                    missing: MissingAddons($missing.get(stringify!($lang)).cloned().unwrap_or_default())
                });
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

#[derive(Serialize)]
pub struct MissingAddons(Vec<String>);

impl std::fmt::Display for MissingAddons {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            write!(f, "None")
        } else if self.0.len() <= 3 {
            write!(f, "{}", self.0.join(", "))
        } else {
            write!(f, "{} and {} more", self.0[0], self.0.len() - 1)
        }
    }
}

#[derive(Serialize)]
pub struct Percentage(f64);

impl std::fmt::Display for Percentage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}%", self.0)
    }
}

#[derive(Tabled, Serialize)]
struct Entry {
    #[tabled(rename = "Language")]
    language: String,
    #[tabled(rename = "Coverage %")]
    percent: Percentage,
    #[tabled(rename = "Addons")]
    missing: MissingAddons,
}

/// Generate a coverage report
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If json serialization fails
pub fn coverage(cmd: &Command) -> Result<Report, Error> {
    let ctx = Context::new(None, crate::context::PreservePrevious::Remove, true)?;

    let mut global = Totals::default();
    let mut missing = HashMap::new();

    let mut table = Vec::new();

    for addon in ctx.addons() {
        let stringtable_path = ctx
            .workspace_path()
            .join(addon.folder())?
            .join("stringtable.xml")?;
        if stringtable_path.exists()? {
            let project = match Project::read(stringtable_path) {
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

    match cmd.format {
        TableFormat::Ascii => {
            println!("{}", Table::new(&table).with(Style::modern()));
        }
        TableFormat::Json => {
            println!(
                "{}",
                serde_json::to_string(&table).expect("Failed to print json")
            );
        }
        TableFormat::PrettyJson => {
            println!(
                "{}",
                serde_json::to_string_pretty(&table).expect("Failed to print json")
            );
        }
        TableFormat::Markdown => {
            println!("{}", Table::new(&table).with(Style::markdown()));
        }
    }

    Ok(Report::new())
}
