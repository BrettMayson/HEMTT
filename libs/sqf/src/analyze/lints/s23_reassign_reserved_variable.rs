use std::{collections::HashMap, ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Label, Processed, Severity}, WorkspacePath,
};

use crate::{analyze::LintData, BinaryCommand, Expression, Statement, UnaryCommand};

crate::analyze::lint!(LintS23ReassignReservedVariable);

impl Lint<LintData> for LintS23ReassignReservedVariable {
    fn ident(&self) -> &'static str {
        "reasign_reserved_variable"
    }

    fn sort(&self) -> u32 {
        230
    }

    fn description(&self) -> &'static str {
        "Prevents reassigning reserved variables"
    }

    fn documentation(&self) -> &'static str {
        r"### Example

**Incorrect**
```sqf
call {
    _this = 1;
};
```

```sqf
{
    private _forEachIndex = random 5;
} forEach allUnits;
```

### Explanation

Reassigning reserved variables can lead to unintentional behavior.
"
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::error()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(StatementsRunner {}), Box::new(ExpressionRunner {})]
    }
}

static RESERVED: [&str; 8] = [
    "this","_this","_forEachIndex","_exception","_thisScript","_thisFSM","thisList","thisTrigger",
];

struct StatementsRunner {}
impl LintRunner<LintData> for StatementsRunner {
    type Target = crate::Statements;

    #[allow(clippy::significant_drop_tightening)]
    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Self::Target,
        _data: &LintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        
        let mut just_saved: Option<(String, String, Range<usize>)> = None;
        let mut codes: Codes = Vec::new();
        let mut need_to_restore: HashMap<String, (String, Range<usize>, Range<usize>)> = HashMap::new();

        for statement in target.content() {
            let (Statement::AssignGlobal(var, exp, span) | Statement::AssignLocal(var, exp, span)) = statement else {
                just_saved.take();
                continue
            };
    
            if let Some((saved, original, saved_span)) = just_saved.as_ref()
                && saved == var {
                    need_to_restore.insert(original.clone(), (saved.clone(), span.clone(), saved_span.clone()));
                    just_saved.take();
                    continue
                }

            if let Some((saved, original_save, saved_span)) = need_to_restore.get(var) {
                codes.push(Arc::new(CodeS23ReassignReservedVariable::new(Variant::SavedWhileSaved(var.clone(), span.clone(), original_save.clone(), saved.clone(), saved_span.clone()), processed, config.severity())));
            }

            if let Expression::Variable(restoring, _) = exp
                && need_to_restore.remove(restoring).is_some() {
                    continue
                }
    
            if RESERVED.contains(&var.as_str()) {
                codes.push(Arc::new(CodeS23ReassignReservedVariable::new(Variant::Overwrite(var.clone(), span.clone()), processed, config.severity())));
            } else if let Expression::Variable(saved, new_saved_span) = exp
                && RESERVED.contains(&saved.as_str()) {
                    just_saved.replace((saved.clone(), var.clone(), new_saved_span.clone()));
                }
        }

        for (saved, (original, span, saved_span)) in need_to_restore {
            codes.push(Arc::new(CodeS23ReassignReservedVariable::new(Variant::NeverRestored((saved, span.clone()), (original, saved_span.clone())), processed, config.severity())));
        }

        codes
    }
}

struct ExpressionRunner {}
impl LintRunner<LintData> for ExpressionRunner {
    type Target = crate::Expression;

    #[allow(clippy::significant_drop_tightening)]
    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Self::Target,
        _data: &LintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        
        let exp = match target {
            Expression::UnaryCommand(UnaryCommand::Named(cmd), exp, _) |
            Expression::BinaryCommand(BinaryCommand::Named(cmd), _, exp, _) if cmd.to_lowercase() == "params" => exp,
            _ => return Vec::new(),
        };

        if let Expression::Array(values, _) | Expression::ConsumeableArray(values, _) = &**exp {
            for param in values {
                let (name, span) = match &param {
                    Expression::String(name, span, _) => (name, span),
                    Expression::Array(values, _) => {
                        if let Some(Expression::String(name, span, _)) = values.first() {
                            (name, span)
                        } else {
                            continue
                        }
                    }
                    _ => continue,
                };
                if RESERVED.contains(&&**name) {
                    return vec![Arc::new(CodeS23ReassignReservedVariable::new(Variant::Overwrite(name.to_string(), span.clone()), processed, config.severity()))];
                }
            }
        }

        Vec::new()
    }
}

pub enum Variant {
    Overwrite(String, Range<usize>),
    NeverRestored((String, Range<usize>),(String, Range<usize>)),
    SavedWhileSaved(String, Range<usize>, Range<usize>, String, Range<usize>),
}

impl Variant {
    #[must_use]
    pub fn span(&self) -> Range<usize> {
        match self {
            Self::Overwrite(_, span) |
            Self::NeverRestored((_, span), _) |
            Self::SavedWhileSaved(_, span, _, _, _) => span.clone(),
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS23ReassignReservedVariable {
    variant: Variant,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS23ReassignReservedVariable {
    fn ident(&self) -> &'static str {
        "L-S23"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#reasign_reserved_variable")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        match &self.variant {
            Variant::Overwrite(var, _) => format!("Reassigning reserved variable `{var}`"),
            Variant::NeverRestored((saved, _), (original, _)) => format!("Reserved variable `{original}` was never restored after being saved to `{saved}`"),
            Variant::SavedWhileSaved(saved, _, _, original, _) => format!("Holder variable `{saved}` is overwritten before restoring `{original}`"),
        }
    }

    fn label_message(&self) -> String {
        String::new()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS23ReassignReservedVariable {
    #[must_use]
    pub fn new(variant: Variant, processed: &Processed, severity: Severity) -> Self {
        Self {
            variant,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let Some(mut diag) = Diagnostic::from_code_processed(&self, self.variant.span(), processed) else {
            return self
        };
        diag = diag.clear_labels();
        match &self.variant {
            Variant::Overwrite(var, span) => {
                let Some(info) = get_span_info(span.clone(), processed) else {
                    return self;
                };
                diag = diag.with_label(Label::primary(
                    info.0,
                    info.1,
                ).with_message(format!("`{var}` is reserved")));
            }
            Variant::NeverRestored((saved, saved_span), (original, original_span)) => {
                let Some(saved_info) = get_span_info(saved_span.clone(), processed) else {
                    return self;
                };
                let Some(original_info) = get_span_info(original_span.clone(), processed) else {
                    return self;
                };
                diag = diag.with_label(Label::secondary(
                    saved_info.0,
                    saved_info.1,
                ).with_message(format!("`{original}` is modified here"))).with_label(Label::primary(
                    original_info.0,
                    original_info.1,
                ).with_message(format!("`{saved}` is never restored to `{original}`")));
            }
            Variant::SavedWhileSaved(saved, saved_span, changed_span, original, original_span) => {
                let Some(saved_info) = get_span_info(saved_span.clone(), processed) else {
                    return self;
                };
                let Some(changed_span) = get_span_info(changed_span.clone(), processed) else {
                    return self;
                };
                let Some(original_info) = get_span_info(original_span.clone(), processed) else {
                    return self;
                };
                diag = diag.with_label(Label::primary(
                    saved_info.0,
                    saved_info.1,
                ).with_message(format!("`{saved}` is overwritten before restoring `{original}`"))).with_label(Label::secondary(
                    changed_span.0,
                    changed_span.1,
                ).with_message(format!("`{original}` is changed"))).with_label(Label::secondary(
                    original_info.0,
                    original_info.1,
                ).with_message(format!("`{saved}` saves the state of {original}")));
            }
        }
        self.diagnostic = Some(diag);
        self
    }
}

fn get_span_info(span: Range<usize>, processed: &Processed) -> Option<(WorkspacePath, Range<usize>)> {
    let map_start = processed.mapping(span.start)?;
    let map_end = processed.mapping(span.end)?;
    let map_file = processed.source(map_start.source())?;
    Some((
        map_file.0.clone(),
        map_start.original_start()..map_end.original_start(),
    ))
}
