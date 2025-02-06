pub mod macros;

use std::{collections::HashMap, sync::Arc};

use codespan_reporting::diagnostic::Severity;
use hemtt_common::config::{LintConfig, LintConfigOverride, LintEnabled, ProjectConfig};

use crate::reporting::{Code, Codes, Diagnostic, Processed};

pub trait Lint<D>: Sync + Send {
    fn display(&self) -> bool {
        true
    }
    fn ident(&self) -> &'static str;
    fn sort(&self) -> u32 {
        0
    }
    fn doc_ident(&self) -> String {
        format!("{:02}", (self.sort() / 10))
    }
    fn description(&self) -> &'static str;
    fn documentation(&self) -> &'static str;
    fn default_config(&self) -> LintConfig;
    fn minimum_severity(&self) -> Severity {
        self.default_config().severity()
    }
    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<D>>>;
}

#[allow(unused_variables, clippy::module_name_repetitions)]
pub trait LintRunner<D> {
    type Target: std::any::Any;

    fn run(
        &self,
        project: Option<&ProjectConfig>,
        config: &LintConfig,
        processed: Option<&Processed>,
        target: &Self::Target,
        data: &D,
    ) -> Codes {
        vec![]
    }
}

pub trait AnyLintRunner<D> {
    fn run(
        &self,
        project: Option<&ProjectConfig>,
        config: &LintConfig,
        processed: Option<&Processed>,
        target: &dyn std::any::Any,
        data: &D,
    ) -> Codes;
}

impl<T: LintRunner<D>, D> AnyLintRunner<D> for T {
    fn run(
        &self,
        project: Option<&ProjectConfig>,
        config: &LintConfig,
        processed: Option<&Processed>,
        target: &dyn std::any::Any,
        data: &D,
    ) -> Codes {
        target
            .downcast_ref::<T::Target>()
            .map_or_else(std::vec::Vec::new, |target| {
                self.run(project, config, processed, target, data)
            })
    }
}

#[allow(unused_variables, clippy::module_name_repetitions)]
pub trait LintGroupRunner<D> {
    type Target: std::any::Any;

    fn run(
        &self,
        project: Option<&ProjectConfig>,
        config: HashMap<String, LintConfig>,
        processed: Option<&Processed>,
        target: &Self::Target,
        data: &D,
    ) -> Codes {
        vec![]
    }
}

pub trait AnyLintGroupRunner<D> {
    fn run(
        &self,
        project: Option<&ProjectConfig>,
        config: HashMap<String, LintConfig>,
        processed: Option<&Processed>,
        target: &dyn std::any::Any,
        data: &D,
    ) -> Codes;
}

impl<T: LintGroupRunner<D>, D> AnyLintGroupRunner<D> for T {
    fn run(
        &self,
        project: Option<&ProjectConfig>,
        config: HashMap<String, LintConfig>,
        processed: Option<&Processed>,
        target: &dyn std::any::Any,
        data: &D,
    ) -> Codes {
        target
            .downcast_ref::<T::Target>()
            .map_or_else(std::vec::Vec::new, |target| {
                self.run(project, config, processed, target, data)
            })
    }
}

pub type Lints<D> = Vec<Arc<Box<dyn Lint<D>>>>;

#[allow(clippy::module_name_repetitions)]
pub struct LintManager<D> {
    lints: Lints<D>,
    groups: Vec<(Lints<D>, Box<dyn AnyLintGroupRunner<D>>)>,
    configs: HashMap<String, LintConfigOverride>,
}

impl<D> LintManager<D> {
    #[must_use]
    pub fn new(configs: HashMap<String, LintConfigOverride>) -> Self {
        Self {
            lints: vec![],
            groups: vec![],
            configs,
        }
    }

    /// Push a lint into the manager
    ///
    /// # Errors
    /// Returns a list of codes if the lint config is invalid
    pub fn push(
        &mut self,
        lint: Arc<Box<dyn Lint<D>>>,
        default_enabled: bool,
    ) -> Result<(), Codes> {
        let lints: Lints<D> = vec![lint];
        let lints = self.check_and_filter(lints, default_enabled)?;
        self.lints.extend(lints);
        Ok(())
    }

    /// Extend the manager with a list of lints
    ///
    /// # Errors
    /// Returns a list of codes if the lint config is invalid
    pub fn extend(&mut self, lints: Lints<D>, default_enabled: bool) -> Result<(), Codes> {
        let lints = self.check_and_filter(lints, default_enabled)?;
        self.lints.extend(lints);
        Ok(())
    }

    /// Push a group of lints into the manager
    ///
    /// # Errors
    /// Returns a list of codes if the lint config is invalid
    pub fn push_group(
        &mut self,
        lints: Lints<D>,
        runner: Box<dyn AnyLintGroupRunner<D>>,
        default_enabled: bool,
    ) -> Result<(), Codes> {
        let lints = self.check_and_filter(lints, default_enabled)?;
        self.groups.push((lints, runner));
        Ok(())
    }

    /// Check if the lints are valid
    ///
    /// # Errors
    /// Returns a list of lints that are enabled OR codes if the lint config is invalid
    pub fn check_and_filter(&self, lints: Lints<D>, pedantic: bool) -> Result<Lints<D>, Codes> {
        fn enabled(config: &LintConfig, pedantic: bool) -> bool {
            config.enabled() == LintEnabled::Enabled
                || (pedantic && config.enabled() == LintEnabled::Pedantic)
        }
        let mut enabled_lints = vec![];
        let mut errors: Codes = vec![];
        for lint in lints {
            if self.lints.iter().any(|l| l.ident() == lint.ident()) {
                errors.push(Arc::new(InvalidLintConfig {
                    message: format!("Lint `{}` already exists", lint.ident()),
                }));
            }
            let mut config = lint.default_config();
            if let Some(config_override) = self.configs.get(lint.ident()) {
                config = config_override.apply(config);
                if config.severity() < lint.minimum_severity() {
                    errors.push(Arc::new(InvalidLintConfig {
                        message: format!(
                            "Lint `{}` severity is lower than minimum severity of {:?}",
                            lint.ident(),
                            lint.minimum_severity(),
                        ),
                    }));
                }
                if !enabled(&config, pedantic) && lint.minimum_severity() == Severity::Error {
                    errors.push(Arc::new(InvalidLintConfig {
                        message: format!("Lint `{}` cannot be disabled", lint.ident()),
                    }));
                }
            }
            if enabled(&config, pedantic) {
                enabled_lints.push(lint);
            }
        }
        if errors.is_empty() {
            Ok(enabled_lints)
        } else {
            Err(errors)
        }
    }

    pub fn run(
        &self,
        data: &D,
        project: Option<&ProjectConfig>,
        processed: Option<&Processed>,
        target: &dyn std::any::Any,
    ) -> Codes {
        self.lints
            .iter()
            .flat_map(|lint| {
                let config = self
                    .configs
                    .get(lint.ident())
                    .cloned()
                    .map_or_else(|| lint.default_config(), |c| c.apply(lint.default_config()));
                lint.runners()
                    .iter()
                    .flat_map(|runner| runner.run(project, &config, processed, target, data))
                    .collect::<Codes>()
            })
            .chain(self.groups.iter().flat_map(|(lints, runner)| {
                let mut configs = HashMap::new();
                for lint in lints {
                    let config = self
                        .configs
                        .get(lint.ident())
                        .cloned()
                        .map_or_else(|| lint.default_config(), |c| c.apply(lint.default_config()));
                    configs.insert(lint.ident().to_string(), config);
                }
                if configs.is_empty() {
                    return vec![];
                }
                runner.run(project, configs, processed, target, data)
            }))
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
        Some(Diagnostic::from_code(self))
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
    impl Lint<()> for LintA {
        fn ident(&self) -> &'static str {
            "LintA"
        }

        fn description(&self) -> &'static str {
            "LintA"
        }

        fn documentation(&self) -> &'static str {
            "LintA"
        }

        fn default_config(&self) -> LintConfig {
            LintConfig::error()
        }

        fn minimum_severity(&self) -> Severity {
            Severity::Error
        }

        fn runners(&self) -> Vec<Box<dyn AnyLintRunner<()>>> {
            vec![Box::new(LintARunner)]
        }
    }

    struct LintARunner;
    impl LintRunner<()> for LintARunner {
        type Target = TypeA;

        fn run(
            &self,
            _project: Option<&ProjectConfig>,
            _config: &LintConfig,
            _processed: Option<&Processed>,
            _target: &TypeA,
            _data: &(),
        ) -> Codes {
            vec![Arc::new(CodeA)]
        }
    }

    struct LintB;
    impl Lint<()> for LintB {
        fn ident(&self) -> &'static str {
            "LintB"
        }

        fn description(&self) -> &'static str {
            "LintB"
        }

        fn documentation(&self) -> &'static str {
            "LintB"
        }

        fn default_config(&self) -> LintConfig {
            LintConfig::error()
        }

        fn minimum_severity(&self) -> Severity {
            Severity::Error
        }

        fn runners(&self) -> Vec<Box<dyn AnyLintRunner<()>>> {
            vec![Box::new(LintBRunner)]
        }
    }

    struct LintBRunner;
    impl LintRunner<()> for LintBRunner {
        type Target = TypeB;

        fn run(
            &self,
            _project: Option<&ProjectConfig>,
            _config: &LintConfig,
            _processed: Option<&Processed>,
            _target: &TypeB,
            _data: &(),
        ) -> Codes {
            vec![Arc::new(CodeB)]
        }
    }

    #[test]
    fn lint_manager() {
        let manager = LintManager {
            lints: vec![Arc::new(Box::new(LintA)), Arc::new(Box::new(LintB))],
            groups: vec![],
            configs: HashMap::new(),
        };

        let target_a = TypeA;
        let target_b = TypeB;
        let target_c = TypeC;

        let codes = manager.run(&(), None, None, &target_a);
        assert_eq!(codes.len(), 1);
        assert_eq!(codes[0].ident(), "CodeA");

        let codes = manager.run(&(), None, None, &target_b);
        assert_eq!(codes.len(), 1);
        assert_eq!(codes[0].ident(), "CodeB");

        let codes = manager.run(&(), None, None, &target_c);
        assert_eq!(codes.len(), 0);
    }
}
