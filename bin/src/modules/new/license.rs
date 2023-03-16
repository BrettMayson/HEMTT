use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "dist/licenses/"]
pub struct Licenses;

impl Licenses {
    pub fn select(author: &str) -> Option<String> {
        let licenses = vec![
            "Arma Public License Share Alike (APL-SA)",
            "Arma Public License (APL)",
            "Arma Public License No Derivatives (APL-ND)",
            "Apache 2.0",
            "GNU GPL v3",
            "MIT",
            "Unlicense",
            "None",
        ];

        let selection = dialoguer::Select::new()
            .with_prompt("Select a license")
            .items(&licenses)
            .default(0)
            .interact()
            .unwrap();
        if selection == 7 {
            return None;
        }

        let license = match selection {
            0 => Self::get("apl-sa.txt").unwrap(),
            1 => Self::get("apl.txt").unwrap(),
            2 => Self::get("apl-nd.txt").unwrap(),
            3 => Self::get("apache.txt").unwrap(),
            4 => Self::get("gpl-3.0.txt").unwrap(),
            5 => Self::get("mit.txt").unwrap(),
            6 => Self::get("unlicense.txt").unwrap(),
            _ => unreachable!(),
        };

        let license = String::from_utf8(license.data.to_vec()).unwrap();
        Some(license.replace("{author}", author).replace(
            "{year}",
            time::OffsetDateTime::now_utc().year().to_string().as_str(),
        ))
    }
}
