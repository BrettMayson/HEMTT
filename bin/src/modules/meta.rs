use crate::modules::Module;

#[derive(Debug, Default)]
pub struct Meta;

impl Module for Meta {
    fn name(&self) -> &'static str {
        "Meta"
    }

    fn pre_release(&self, ctx: &crate::context::Context) -> Result<crate::report::Report, crate::Error> {
        let path = ctx.build_folder().expect("Failed to get build folder").join("meta.cpp");
        if !path.exists() {
            return Ok(crate::report::Report::new());
        }
        let content = std::fs::read_to_string(&path)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("valid system time")
            .as_secs()
                * 10_000_000
                + 621_355_968_000_000_000; // .net ticks
        let new_content = if content.contains("timestamp") {
            let re = regex::Regex::new(r"timestamp\s*=\s*(\d+);").expect("valid regex");
            re.replace_all(&content, format!("timestamp = {now};")).to_string()
        } else {
            format!("{content}\ntimestamp = {now};\n")
        };
        std::fs::write(&path, new_content)?;
        info!("Updated meta.cpp with current timestamp.");
        Ok(crate::report::Report::new())
    }
}
