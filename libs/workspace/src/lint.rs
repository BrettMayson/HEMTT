use std::{collections::HashMap, sync::Arc};

use codespan_reporting::diagnostic::Severity;
use hemtt_common::config::{LintConfig, ProjectConfig};

use crate::reporting::{Code, Diagnostic, Processed};

pub trait Lint {
    fn ident(&self) -> &str;
    fn description(&self) -> &str;
    fn documentation(&self) -> &str;
    fn default_config(&self) -> LintConfig;
    fn minimum_severity(&self) -> Severity {
        self.default_config().severity()
    }
    fn runners(&self) -> Vec<Box<dyn AnyLintRunner>>;
}

#[allow(unused_variables, clippy::module_name_repetitions)]
pub trait LintRunner {
    type Target: std::any::Any;

    fn run(
        &self,
        project: Option<&ProjectConfig>,
        config: &LintConfig,
        processed: Option<&Processed>,
        target: &Self::Target,
    ) -> Vec<Arc<dyn Code>> {
        vec![]
    }
}

pub trait AnyLintRunner {
    fn run(
        &self,
        project: Option<&ProjectConfig>,
        config: &LintConfig,
        processed: Option<&Processed>,
        target: &dyn std::any::Any,
    ) -> Vec<Arc<dyn Code>>;
}

impl<T: LintRunner> AnyLintRunner for T {
    fn run(
        &self,
        project: Option<&ProjectConfig>,
        config: &LintConfig,
        processed: Option<&Processed>,
        target: &dyn std::any::Any,
    ) -> Vec<Arc<dyn Code>> {
        target
            .downcast_ref::<T::Target>()
            .map_or_else(std::vec::Vec::new, |target| {
                self.run(project, config, processed, target)
            })
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct LintManager {
    lints: Vec<Box<dyn Lint>>,
    configs: HashMap<String, LintConfig>,
}

impl LintManager {
    #[must_use]
    pub fn new(configs: HashMap<String, LintConfig>) -> Self {
        Self {
            lints: vec![],
            configs,
        }
    }

    /// Push a lint into the manager
    ///
    /// # Errors
    /// Returns a list of codes if the lint config is invalid
    pub fn push(&mut self, lint: Box<dyn Lint>) -> Result<(), Vec<Arc<dyn Code>>> {
        let lints = vec![lint];
        self.check(&lints)?;
        self.lints.extend(lints);
        Ok(())
    }

    /// Extend the manager with a list of lints
    ///
    /// # Errors
    /// Returns a list of codes if the lint config is invalid
    pub fn extend(&mut self, lints: Vec<Box<dyn Lint>>) -> Result<(), Vec<Arc<dyn Code>>> {
        self.check(&lints)?;
        self.lints.extend(lints);
        Ok(())
    }

    /// Check if the lints are valid
    ///
    /// # Errors
    /// Returns a list of codes if the lint config is invalid
    pub fn check(&self, lints: &[Box<dyn Lint>]) -> Result<(), Vec<Arc<dyn Code>>> {
        let mut errors: Vec<Arc<dyn Code>> = vec![];
        for lint in lints {
            if self.lints.iter().any(|l| l.ident() == lint.ident()) {
                errors.push(Arc::new(InvalidLintConfig {
                    message: format!("Lint `{}` already exists", lint.ident()),
                }));
            }
            if let Some(config) = self.configs.get(lint.ident()) {
                if config.severity() < lint.minimum_severity() {
                    errors.push(Arc::new(InvalidLintConfig {
                        message: format!(
                            "Lint `{}` severity is lower than minimum severity of {:?}",
                            lint.ident(),
                            lint.minimum_severity(),
                        ),
                    }));
                }
                if !config.enabled() && lint.minimum_severity() == Severity::Error {
                    errors.push(Arc::new(InvalidLintConfig {
                        message: format!("Lint `{}` cannot be disabled", lint.ident()),
                    }));
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn run(
        &self,
        project: Option<&ProjectConfig>,
        processed: Option<&Processed>,
        target: &dyn std::any::Any,
    ) -> Vec<Arc<dyn Code>> {
        self.lints
            .iter()
            .flat_map(|lint| {
                let config = self
                    .configs
                    .get(lint.ident())
                    .cloned()
                    .unwrap_or_else(|| lint.default_config());
                if !config.enabled() {
                    return vec![];
                }
                lint.runners()
                    .iter()
                    .flat_map(|runner| runner.run(project, &config, processed, target))
                    .collect::<Vec<Arc<dyn Code>>>()
            })
            .collect()
    }
}

struct InvalidLintConfig {
    message: String,
}
impl Code for InvalidLintConfig {
    fn ident(&self) -> &'static str {
        "ILC"
    }

    fn message(&self) -> String {
        self.message.clone()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::simple(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TypeA;
    struct TypeB;
    struct TypeC;

    struct CodeA;
    impl Code for CodeA {
        fn ident(&self) -> &'static str {
            "CodeA"
        }

        fn message(&self) -> String {
            "CodeA".to_string()
        }

        fn label_message(&self) -> String {
            "CodeA".to_string()
        }

        fn help(&self) -> Option<String> {
            Some("CodeA".to_string())
        }
    }

    struct CodeB;
    impl Code for CodeB {
        fn ident(&self) -> &'static str {
            "CodeB"
        }

        fn message(&self) -> String {
            "CodeB".to_string()
        }

        fn label_message(&self) -> String {
            "CodeB".to_string()
        }

        fn help(&self) -> Option<String> {
            Some("CodeB".to_string())
        }
    }

    struct LintA;
    impl Lint for LintA {
        fn ident(&self) -> &str {
            "LintA"
        }

        fn description(&self) -> &str {
            "LintA"
        }

        fn documentation(&self) -> &str {
            "LintA"
        }

        fn default_config(&self) -> LintConfig {
            LintConfig::error()
        }

        fn minimum_severity(&self) -> Severity {
            Severity::Error
        }

        fn runners(&self) -> Vec<Box<dyn AnyLintRunner>> {
            vec![Box::new(LintARunner)]
        }
    }

    struct LintARunner;
    impl LintRunner for LintARunner {
        type Target = TypeA;

        fn run(
            &self,
            _project: Option<&ProjectConfig>,
            _config: &LintConfig,
            _processed: Option<&Processed>,
            _target: &TypeA,
        ) -> Vec<Arc<dyn Code>> {
            vec![Arc::new(CodeA)]
        }
    }

    struct LintB;
    impl Lint for LintB {
        fn ident(&self) -> &str {
            "LintB"
        }

        fn description(&self) -> &str {
            "LintB"
        }

        fn documentation(&self) -> &str {
            "LintB"
        }

        fn default_config(&self) -> LintConfig {
            LintConfig::error()
        }

        fn minimum_severity(&self) -> Severity {
            Severity::Error
        }

        fn runners(&self) -> Vec<Box<dyn AnyLintRunner>> {
            vec![Box::new(LintBRunner)]
        }
    }

    struct LintBRunner;
    impl LintRunner for LintBRunner {
        type Target = TypeB;

        fn run(
            &self,
            _project: Option<&ProjectConfig>,
            _config: &LintConfig,
            _processed: Option<&Processed>,
            _target: &TypeB,
        ) -> Vec<Arc<dyn Code>> {
            vec![Arc::new(CodeB)]
        }
    }

    #[test]
    fn test_lint_manager() {
        let manager = LintManager {
            lints: vec![Box::new(LintA), Box::new(LintB)],
            configs: HashMap::new(),
        };

        let target_a = TypeA;
        let target_b = TypeB;
        let target_c = TypeC;

        let codes = manager.run(None, None, &target_a);
        assert_eq!(codes.len(), 1);
        assert_eq!(codes[0].ident(), "CodeA");

        let codes = manager.run(None, None, &target_b);
        assert_eq!(codes.len(), 1);
        assert_eq!(codes[0].ident(), "CodeB");

        let codes = manager.run(None, None, &target_c);
        assert_eq!(codes.len(), 0);
    }
}
