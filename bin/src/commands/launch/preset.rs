use hemtt_common::arma::dlc::DLC;
use regex::Regex;

/// Read a preset file and return the mods and DLCs
///
/// # Panics
/// Will panic if the regex can not be compiled, which should never be the case in a released version
pub fn read(name: &str, html: &str) -> (Vec<String>, Vec<DLC>) {
    let mut workshop = Vec::new();
    let mut dlc = Vec::new();
    let mod_regex = Regex::new(
        r#"(?m)href="https?:\/\/steamcommunity\.com\/sharedfiles\/filedetails\/\?id=(\d+)""#,
    )
    .expect("mod regex compiles");
    for id in mod_regex.captures_iter(html).map(|c| c[1].to_string()) {
        if workshop.contains(&id) {
            trace!("Skipping mod {} in preset {}", id, name);
        } else {
            trace!("Found new mod {} in preset {}", id, name);
            workshop.push(id);
        }
    }
    let dlc_regex = Regex::new(r#"(?m)href="https?:\/\/store\.steampowered\.com\/app\/(\d+)""#)
        .expect("dlc regex compiles");
    for id in dlc_regex.captures_iter(html).map(|c| c[1].to_string()) {
        let Ok(preset_dlc) = DLC::try_from(id.clone()) else {
            warn!(
                "Preset {} requires DLC {}, but HEMTT does not recognize it",
                name, id
            );
            continue;
        };
        if dlc.contains(&preset_dlc) {
            trace!("Skipping DLC {} in preset {}", id, name);
        } else {
            trace!("Found new DLC {} in preset {}", id, name);
            dlc.push(preset_dlc);
        }
    }
    (workshop, dlc)
}
