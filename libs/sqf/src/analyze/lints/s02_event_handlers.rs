use std::{ops::Range, sync::Arc};

use arma3_wiki::model::{EventHandlerNamespace, ParsedEventHandler, Version};
use hemtt_common::{config::{LintConfig, ProjectConfig}, similar_values};
use hemtt_workspace::{
    addons::Addon, lint::{AnyLintRunner, Lint, LintGroupRunner}, reporting::{Code, Codes, Diagnostic, Label, Processed, Severity}, WorkspacePath
};

use crate::{analyze::{extract_constant, LintData}, parser::database::Database, BinaryCommand, Expression, Statements, UnaryCommand};

pub struct LintS02EventInsufficientVersion;

impl Lint<LintData> for LintS02EventInsufficientVersion {
    fn ident(&self) -> &'static str {
        "event_insufficient_version"
    }

    fn sort(&self) -> u32 {
        20
    }

    fn doc_ident(&self) -> String {
        "02IV".to_string()
    }

    fn description(&self) -> &'static str {
        "Checks for event handlers that require a newer version than specified in CfgPatches"
    }

    fn documentation(&self) -> &'static str {
r#"### Example

**Incorrect**
```hpp
class CfgPatches {
    class MyAddon {
        units[] = {};
        weapons[] = {};
        requiredVersion = 2.00;
    };
};
```
```sqf
_this addEventHandler ["OpticsModeChanged", { // Requires 2.10
    hint 'Optics mode changed';
}];
```

Check [the wiki](https://community.bistudio.com/wiki/Arma_3:_Event_Handlers) to see what in version events were introduced.
"#
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::error()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![]
    }
}

pub struct LintS02EventUnknown;

impl Lint<LintData> for LintS02EventUnknown {
    fn ident(&self) -> &'static str {
        "event_unknown"
    }

    fn sort(&self) -> u32 {
        21
    }

    fn doc_ident(&self) -> String {
        "02UE".to_string()
    }

    fn description(&self) -> &'static str {
        "Checks for unknown event used in event handlers"
    }

    fn documentation(&self) -> &'static str {
r#"### Configuration

- **ignore**: List of unknown event names to ignore

```toml
[lints.sqf.event_unknown]
options.ignore = [
    "HealingReceived",
]
```

### Example

**Incorrect**
```sqf
_this addEventHandler ["HealingReceived", { // HealingReceived is not a valid event
    hint 'Healing received';
}];
```

Check [the wiki](https://community.bistudio.com/wiki/Arma_3:_Event_Handlers) to see what events are available.
"#
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::warning()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![]
    }
}

pub struct LintS02EventIncorrectCommand;

impl Lint<LintData> for LintS02EventIncorrectCommand {
    fn ident(&self) -> &'static str {
        "event_incorrect_command"
    }

    fn sort(&self) -> u32 {
        22
    }

    fn doc_ident(&self) -> String {
        "02IC".to_string()
    }

    fn description(&self) -> &'static str {
        "Checks for event handlers used with incorrect commands"
    }

    fn documentation(&self) -> &'static str {
r#"### Example

**Incorrect**
```sqf
_this addEventHandler ["MPHit", {
    hint 'Hit';
}];
```
**Correct**
```sqf
_this addMPEventHandler ["MPHit", {
    hint 'Hit';
}];
```
"#
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::error()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![]
    }
}

pub struct EventHandlerRunner;
impl LintGroupRunner<LintData> for EventHandlerRunner {
    type Target = Statements;
    fn run(
        &self,
        _project: Option<&ProjectConfig>,
        _build_info: Option<&hemtt_common::config::BuildInfo>,
        config: std::collections::HashMap<String, LintConfig>,
        processed: Option<&Processed>,
        target: &Statements,
        data: &LintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        let mut codes: Codes = Vec::new();
        let (addon, database) = data;
        for statement in target.content() {
            for expression in statement.walk_expressions() {
                let Some((ns, name, id, target)) = get_namespaces(expression) else {
                    continue;
                };
                if ns.is_empty() {
                    continue;
                }
                if name.contains("UserAction") {
                    // Requires arma3-wiki to parse and provide https://community.bistudio.com/wiki/inputAction/actions
                    continue;
                }
                let eh = database.wiki().event_handler(&id.0);
                codes.extend(check_unknown(
                    &ns,
                    &name,
                    &id,
                    target.map(|t| &**t),
                    &eh,
                    processed,
                    database,
                    config.get("event_unknown"),
                ));
                codes.extend(check_version(
                    addon, &ns, &name, &id, &eh, processed, database,
                ));
            }
        }
        codes
    }
}

#[allow(clippy::too_many_arguments)]
fn check_unknown(
    ns: &[EventHandlerNamespace],
    name: &str,
    id: &(Arc<str>, &Range<usize>),
    target: Option<&Expression>,
    eh: &[(EventHandlerNamespace, &ParsedEventHandler)],
    processed: &Processed,
    database: &Database,
    config: Option<&LintConfig>,
) -> Codes {
    if let Some(config) = config {
        if !config.enabled() {
            return Vec::new();
        }
    }

    if let Some(config) = config {
        if let Some(toml::Value::Array(ignore)) = config.option("ignore") {
            if ignore.iter().any(|i| i.as_str() == Some(name)) {
                return Vec::new();
            }
        }
    }

    if eh.is_empty() {
        return vec![Arc::new(CodeS02UnknownEvent::new(
            ns,
            id.1.clone(),
            name.to_owned(),
            id.0.clone(),
            processed,
            database,
            config.map_or(Severity::Warning, LintConfig::severity),
        ))];
    }

    if ns.iter().any(|n| eh.iter().any(|(ns, _)| ns == n)) {
        return Vec::new();
    }
    vec![Arc::new(CodeS02IncorrectCommand::new(
        id.1.clone(),
        name.to_owned(),
        id.0.clone(),
        target.and_then(extract_constant),
        eh.iter().map(|(ns, _)| ns).copied().collect(),
        processed,
        database,
    ))]
}

fn check_version(
    addon: &Addon,
    ns: &[EventHandlerNamespace],
    name: &str,
    id: &(Arc<str>, &Range<usize>),
    eh: &[(EventHandlerNamespace, &ParsedEventHandler)],
    processed: &Processed,
    database: &Database,
) -> Codes {
    let Some(required) = addon.build_data().required_version() else {
        // TODO what to do here?
        return Vec::new();
    };
    let Some((_, eh)) = eh.iter().find(|(ins, _)| ns.contains(ins)) else {
        return Vec::new();
    };
    let Some(since) = eh.since() else {
        return Vec::new();
    };
    let Some(version) = since.arma_3() else {
        return Vec::new();
    };
    let mut errors: Codes = Vec::new();
    let wiki_version = arma3_wiki::model::Version::new(
        u8::try_from(required.0.major()).unwrap_or_default(),
        u8::try_from(required.0.minor()).unwrap_or_default(),
    );
    let required = (wiki_version, required.1, required.2);
    if wiki_version < *version {
        errors.push(Arc::new(CodeS02InsufficientVersion::new(
            name.to_owned(),
            id.1.clone(),
            *version,
            required,
            *database.wiki().version(),
            processed,
        )));
    }
    errors
}

#[allow(clippy::type_complexity)]
fn get_namespaces(
    expression: &Expression,
) -> Option<(
    Vec<EventHandlerNamespace>,
    String,
    (Arc<str>, &Range<usize>),
    Option<&Box<Expression>>,
)> {
    match expression {
        Expression::BinaryCommand(BinaryCommand::Named(name), target, id, _) => Some((
            EventHandlerNamespace::by_command(name),
            name.to_owned(),
            get_id(id)?,
            Some(target),
        )),
        Expression::UnaryCommand(UnaryCommand::Named(name), id, _) => Some((
            EventHandlerNamespace::by_command(name),
            name.to_owned(),
            get_id(id)?,
            None,
        )),
        _ => None,
    }
}

fn get_id(expression: &Expression) -> Option<(Arc<str>, &Range<usize>)> {
    match expression {
        Expression::String(id, span, _) => Some((id.clone(), span)),
        Expression::Array(items, _) => {
            if items.is_empty() {
                None
            } else {
                get_id(&items[0])
            }
        }
        _ => None,
    }
}

pub struct CodeS02UnknownEvent {
    span: Range<usize>,
    command: String,
    id: Arc<str>,

    similar: Vec<String>,

    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS02UnknownEvent {
    fn ident(&self) -> &'static str {
        "L-S02UE"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#event_unknown")
    }

    fn severity(&self) -> Severity {
        if self.id.to_lowercase() == "damaged" {
            Severity::Error
        } else {
            self.severity
        }
    }

    fn message(&self) -> String {
        format!("Using `{}` with unknown event `{}`", self.command, self.id)
    }

    fn label_message(&self) -> String {
        format!("unknown event `{}`", self.id)
    }

    fn help(&self) -> Option<String> {
        if self.similar.is_empty() {
            None
        } else {
            Some(format!("Did you mean: `{}`?", self.similar.join("`, `")))
        }
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone().map(|d| {
            if self.id.to_lowercase() == "damaged" {
                d.with_help("Damaged is a common typo for `Dammaged`. An error has been raised to prevent accidental usage.")
            } else {
                d
            }
        })
    }
}

impl CodeS02UnknownEvent {
    #[must_use]
    pub fn new(
        nss: &[EventHandlerNamespace],
        span: Range<usize>,
        command: String,
        id: Arc<str>,
        processed: &Processed,
        database: &Database,
        severity: Severity,
    ) -> Self {
        Self {
            span,
            command,

            similar: {
                let mut haystack = Vec::new();
                for (dns, ehs) in database.wiki().event_handlers() {
                    if !nss.contains(dns) {
                        continue;
                    }
                    for eh in ehs {
                        haystack.push(eh.id());
                    }
                }
                let mut similar: Vec<String> = similar_values(&id, &haystack)
                    .into_iter()
                    .map(std::string::ToString::to_string)
                    .collect();
                similar.sort();
                similar.dedup();
                similar
            },

            id,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}

pub struct CodeS02IncorrectCommand {
    span: Range<usize>,
    command: String,
    id: Arc<str>,
    target: Option<(String, bool)>,

    alternatives: Vec<(String, bool)>,

    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS02IncorrectCommand {
    fn ident(&self) -> &'static str {
        "L-S02IC"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#event_incorrect_command")
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn message(&self) -> String {
        format!(
            "Event `{}` was not expected for command `{}`",
            self.id, self.command
        )
    }

    fn label_message(&self) -> String {
        format!("not supported by command `{}`", self.command)
    }

    fn suggestion(&self) -> Option<String> {
        if self.alternatives.len() == 1 {
            if self.alternatives[0].1 {
                if let Some((target, _)) = &self.target {
                    Some(format!(
                        "{} {} [\"{}\", {{ …",
                        target, self.alternatives[0].0, self.id
                    ))
                } else {
                    #[allow(clippy::literal_string_with_formatting_args)]
                    Some(format!(
                        "{{target}} {} [\"{}\", {{ …",
                        self.alternatives[0].0, self.id
                    ))
                }
            } else {
                Some(self.alternatives[0].0.clone())
            }
        } else {
            None
        }
    }

    fn help(&self) -> Option<String> {
        if self.alternatives.is_empty() {
            None
        } else {
            Some(format!(
                "Did you mean: `{}`?",
                self.alternatives
                    .iter()
                    .map(|(a, _)| a.as_str())
                    .collect::<Vec<_>>()
                    .join("`, `")
            ))
        }
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS02IncorrectCommand {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        command: String,
        id: Arc<str>,
        target: Option<(String, bool)>,
        namespaces: Vec<EventHandlerNamespace>,
        processed: &Processed,
        database: &Database,
    ) -> Self {
        let prefix = command.chars().take(3).collect::<String>();
        Self {
            span,
            command,
            id,
            target,
            alternatives: {
                let mut alternatives = Vec::new();
                for ns in namespaces {
                    println!("Possible alternatives: {:?}", ns.commands());
                    ns.commands()
                        .iter()
                        .filter(|c| c.contains(&prefix))
                        .for_each(|c| {
                            alternatives.push(((*c).to_string(), {
                                database.wiki().commands().get(c).is_some_and(|c| {
                                    c.syntax().first().is_some_and(|s| s.call().is_binary())
                                })
                            }));
                        });
                }
                alternatives.sort();
                alternatives.dedup();
                alternatives
            },
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}

pub struct CodeS02InsufficientVersion {
    event: String,
    span: Range<usize>,
    version: Version,
    required: (Option<Version>, WorkspacePath, Range<usize>),
    stable: Version,

    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS02InsufficientVersion {
    fn ident(&self) -> &'static str {
        "L-S02IV"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#event_insufficient_version")
    }

    fn message(&self) -> String {
        format!("event `{}` requires version {}", self.event, self.version)
    }

    fn label_message(&self) -> String {
        format!("requires version {}", self.version)
    }

    fn note(&self) -> Option<String> {
        if self.version > self.stable {
            Some(format!(
                "Current stable version is {}. Using {} will require the development branch.",
                self.stable, self.version
            ))
        } else {
            None
        }
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS02InsufficientVersion {
    #[must_use]
    pub fn new(
        event: String,
        span: Range<usize>,
        version: Version,
        required: (Version, WorkspacePath, Range<usize>),
        stable: Version,
        processed: &Processed,
    ) -> Self {
        Self {
            event,
            span,
            version,
            required: {
                if required.0.major() == 0 && required.0.minor() == 0 {
                    (None, required.1, required.2)
                } else {
                    (Some(required.0), required.1, required.2)
                }
            },
            stable,

            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let Some(diag) = Diagnostic::from_code_processed(&self, self.span.clone(), processed) else {
            return self;
        };
        self.diagnostic = Some(diag.with_label(
            Label::secondary(self.required.1.clone(), self.required.2.clone()).with_message(
                self.required.0.map_or_else(
                    || "CfgPatches entry doesn't specify `requiredVersion`".to_string(),
                    |required| format!("CfgPatches entry requires version {required}"),
                ),
            ),
        ));
        self
    }
}
