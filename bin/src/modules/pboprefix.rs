use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::modules::Module;

#[derive(Debug, Default)]
pub struct PboPrefix;

impl Module for PboPrefix {
    fn name(&self) -> &'static str {
        "pboprefix"
    }
    fn pre_build(
        &self,
        ctx: &crate::context::Context,
    ) -> Result<crate::report::Report, crate::Error> {
        let addons = ctx.addons().iter().collect::<Vec<_>>();
        let project_expected = ctx.config().expected_path(); // e.g. "z\ace\"
        trace!(
            "Checking prefixes against expected project path: {}",
            project_expected
        );
        addons.par_iter().for_each(|addon| {
            if let Some(config) = addon.config()
                && config.ignore_pboprefix()
            {
                return;
            }
            let addon_expected = format!("{}addons\\{}", project_expected, addon.name());
            if !addon
                .prefix()
                .to_string()
                .eq_ignore_ascii_case(&addon_expected)
            {
                warn!("Unexpected pboprefix for addon: `{}`", addon.name());
            }
        });
        Ok(crate::report::Report::new())
    }
}
