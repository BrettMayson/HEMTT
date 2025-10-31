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
            Some(license.replace("{AUTHOR}", author).replace(
                "{YEAR}",
                time::OffsetDateTime::now_utc().year().to_string().as_str(),
            ))
        }
    }

    /// Get a license by name
    ///
    /// # Arguments
    /// * `name` - The license name (e.g., "apl-sa", "mit", "apache")
    /// * `author` - The author name to include in the license
    ///
    /// # Returns
    /// The license text with author and year filled in, or None if the license name is invalid
    #[must_use]
    pub fn get_by_name(name: &str, author: &str) -> Option<String> {
        let license = match name.to_lowercase().as_str() {
            "apl-sa" => include_str!("licenses/apl-sa.txt"),
            "apl" => include_str!("licenses/apl.txt"),
            "apl-nd" => include_str!("licenses/apl-nd.txt"),
            "apache" | "apache-2.0" | "apache2" => include_str!("licenses/apache.txt"),
            "gpl" | "gpl-3.0" | "gpl3" => include_str!("licenses/gpl-3.0.txt"),
            "mit" => include_str!("licenses/mit.txt"),
            "unlicense" => include_str!("licenses/unlicense.txt"),
            _ => return None,
        };
        #[allow(clippy::literal_string_with_formatting_args)]
        {
            Some(license.replace("{AUTHOR}", author).replace(
                "{YEAR}",
                time::OffsetDateTime::now_utc().year().to_string().as_str(),
            ))
        }
    }

    /// Write a license file to the given path
    ///
    /// # Errors
    /// Returns an error if the file cannot be created or written to
    pub fn write_license_file(
        license_text: &str,
        path: &std::path::Path,
    ) -> Result<(), std::io::Error> {
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;
        file.write_all(license_text.as_bytes())?;
        Ok(())
    }
}
