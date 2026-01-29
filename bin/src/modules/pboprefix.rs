use hemtt_workspace::reporting::{Code, Severity};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::sync::Arc;

use crate::modules::Module;

#[derive(Debug, Default)]
pub struct PboPrefix;

impl Module for PboPrefix {
    fn name(&self) -> &'static str {
        "pboprefix"
    }
    fn check(&self, ctx: &crate::context::Context) -> Result<crate::report::Report, crate::Error> {
        let addons = ctx.addons().iter().collect::<Vec<_>>();
        let project_expected = ctx.config().expected_path(); // e.g. "z\ace\"
        trace!(
            "Checking prefixes against expected project path: {}",
            project_expected
        );
        let results = addons
            .par_iter()
            .map(|addon| -> Option<Arc<dyn Code>> {
                if let Some(config) = addon.config()
                    && config.ignore_pboprefix()
                {
                    return None;
                }
                let addon_expected = format!("{}addons\\{}", project_expected, addon.name());
                if addon
                    .prefix()
                    .to_string()
                    .eq_ignore_ascii_case(&addon_expected)
                {
                    None
                } else {
                    Some(std::sync::Arc::new(CodePboPrefixInvalid::new(
                        addon.name().to_string(),
                        addon_expected,
                    )))
                }
            })
            .filter_map(|x| x)
            .collect::<Vec<_>>();
        let mut report = crate::report::Report::new();
        report.extend(results);
        Ok(report)
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodePboPrefixInvalid {
    addon: String,
    expected: String,
}

impl Code for CodePboPrefixInvalid {
    fn ident(&self) -> &'static str {
        "INVALID-PBOPREFIX"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn message(&self) -> String {
        format!("Unexpected pboprefix for addon: `{}`", self.addon)
    }
    fn help(&self) -> Option<String> {
        Some(format!("Expected prefix: `{}`", self.expected))
    }
}

impl CodePboPrefixInvalid {
    #[must_use]
    pub const fn new(addon: String, expected: String) -> Self {
        Self { addon, expected }
    }
}
