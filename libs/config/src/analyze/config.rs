use hemtt_error::{processed::Processed, Code};

use crate::Config;

use super::Analyze;

impl Analyze for Config {
    fn valid(&self) -> bool {
        self.0.iter().all(Analyze::valid)
    }

    fn warnings(&self, processed: &Processed) -> Vec<Box<dyn Code>> {
        self.0
            .iter()
            .flat_map(|p| p.warnings(processed))
            .collect::<Vec<_>>()
    }

    fn errors(&self, processed: &Processed) -> Vec<Box<dyn Code>> {
        self.0
            .iter()
            .flat_map(|p| p.errors(processed))
            .collect::<Vec<_>>()
    }
}
