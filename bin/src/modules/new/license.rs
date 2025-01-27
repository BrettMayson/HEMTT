pub struct Licenses;

impl Licenses {
    #[must_use]
    /// Has the user select a license
    ///
    /// # Panics
    /// If there is a problem with dialoguer
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
            .expect("Failed to get license selection");
        if selection == 7 {
            return None;
        }

        let license = match selection {
            0 => include_str!("licenses/apl-sa.txt"),
            1 => include_str!("licenses/apl.txt"),
            2 => include_str!("licenses/apl-nd.txt"),
            3 => include_str!("licenses/apache.txt"),
            4 => include_str!("licenses/gpl-3.0.txt"),
            5 => include_str!("licenses/mit.txt"),
            6 => include_str!("licenses/unlicense.txt"),
            _ => unreachable!(),
        };
        #[allow(clippy::literal_string_with_formatting_args)]
        {
            Some(license.replace("{author}", author).replace(
                "{year}",
                time::OffsetDateTime::now_utc().year().to_string().as_str(),
            ))
        }
    }
}
