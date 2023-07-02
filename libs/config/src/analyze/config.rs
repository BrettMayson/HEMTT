use crate::Config;

use super::Analyze;

impl Analyze for Config {
    fn valid(&self) -> bool {
        self.0.iter().all(Analyze::valid)
    }

    fn warnings(&self, processed: &hemtt_preprocessor::Processed) -> Vec<String> {
        self.0
            .iter()
            .flat_map(|p| p.warnings(processed))
            .collect::<Vec<_>>()
    }

    fn errors(&self, processed: &hemtt_preprocessor::Processed) -> Vec<String> {
        self.0
            .iter()
            .flat_map(|p| p.errors(processed))
            .collect::<Vec<_>>()
    }
}
