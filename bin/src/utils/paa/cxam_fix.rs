use std::io::{Seek, SeekFrom};

use crate::Error;

#[derive(clap::Parser)]
/// Fix PAAs with incorrect CXAM (max color) tagg values
///
/// This was due to a bug in HEMTT v1.18.1 and earlier where the CXAM tagg
/// was calculated when converting images to PAA format.
pub struct Command {}

/// Execute the final newline command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If the args are not present from clap
pub fn execute(_: &Command) -> Result<(), Error> {
    let mut count = 0;
    for entry in walkdir::WalkDir::new(".") {
        let entry = entry?;
        let path = entry.path();
        if path.display().to_string().contains(".hemttout") {
            continue;
        }
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        if ext != "paa" {
            continue;
        }
        let mut file = fs_err::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;
        let mut paa = hemtt_paa::Paa::read(&mut file)?;
        let Some(cxam) = paa.taggs().get("CXAM") else {
            continue;
        };
        if cxam.len() == 4 {
            let max_color = [cxam[0], cxam[1], cxam[2], cxam[3]];
            if max_color != [255, 255, 255, 255] {
                paa.fix_cxam_tagg();
                file.seek(SeekFrom::Start(0))?;
                paa.write(&mut file)?;
                info!("Fixed CXAM tagg in {}", path.display());
                count += 1;
            }
        }
    }

    info!("Fixed CXAM tagg values in {} PAA files", count);

    Ok(())
}
