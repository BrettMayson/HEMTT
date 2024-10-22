#[macro_export]
macro_rules! lint_manager {
    ($ident:ident, $groups:expr) => {
        $crate::paste::paste! {
            #[linkme::distributed_slice]
            pub static [<$ident:upper _LINTS>]: [std::sync::LazyLock<
                std::sync::Arc<Box<dyn hemtt_workspace::lint::Lint<super::analyze::SqfLintData>>>,
            >];

            #[allow(unused_macros)]
            macro_rules! lint {
                ($name:ident) => {
                    #[allow(clippy::module_name_repetitions)]
                    pub struct $name;
                    #[linkme::distributed_slice(super::super::[<$ident:upper _LINTS>])]
                    static LINT_ADD: std::sync::LazyLock<
                        std::sync::Arc<Box<dyn hemtt_workspace::lint::Lint<super::super::SqfLintData>>>,
                    > = std::sync::LazyLock::new(|| std::sync::Arc::new(Box::new($name)));
                };
            }
            pub(crate) use lint;

            #[must_use]
            pub fn lint_check(
                config: std::collections::HashMap<String, hemtt_common::config::LintConfigOverride>,
            ) -> $crate::reporting::Codes {
                let mut manager: $crate::lint::LintManager<super::analyze::SqfLintData> =
                    $crate::lint::LintManager::new(config);
                if let Err(lint_errors) =
                    manager.extend([<$ident:upper _LINTS>].iter().map(|l| (**l).clone()).collect::<Vec<_>>())
                {
                    return lint_errors;
                }
                let groups: Vec<(
                    $crate::lint::Lints<super::analyze::SqfLintData>,
                    Box<dyn $crate::lint::AnyLintGroupRunner<super::analyze::SqfLintData>>,
                )> = $groups;
                for group in groups {
                    if let Err(lint_errors) = manager.push_group(group.0, group.1) {
                        return lint_errors;
                    }
                }
                vec![]
            }
        }
    };
}
