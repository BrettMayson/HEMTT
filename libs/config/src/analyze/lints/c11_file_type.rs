use std::{ops::Range, sync::Arc};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Diagnostic, Processed, Severity},
};

use crate::{analyze::LintData, Item, Property, Str, Value};

crate::analyze::lint!(LintC11FileType);

impl Lint<LintData> for LintC11FileType {
    fn ident(&self) -> &'static str {
        "file_type"
    }

    fn sort(&self) -> u32 {
        110
    }

    fn description(&self) -> &'static str {
        "Reports on properties that have an unusual or missing file type"
    }

    fn documentation(&self) -> &'static str {
r#"### Configuration

- **allow_no_extension**: Allow properties to not have a file extension, default is `false`.

```toml
[lints.config.file_type]
options.allow_no_extension = true
```

### Example

**Incorrect**
```hpp
class MyClass {
    model = "model.blend";
};
```

**Correct**
```hpp
class MyClass {
    model = "model.p3d";
};
```

**Incorrect**
```hpp
class MyClass {
    editorPreview = "preview.jgp";
}
```

**Correct**
```hpp
class MyClass {
    editorPreview = "preview.jpg";
};
```

**Incorrect, when `allow_no_extension` is `false`**
```hpp
class MyClass {
    model = "my_model";
};
```

**Correct**
```hpp
class MyClass {
    model = "my_model.p3d";
};
```

### Explanation

Some properties require a specific file type. This lint will report on properties that have an unusual file type, from typos or incorrect file extensions.
"#
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::warning()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;

impl LintRunner<LintData> for Runner {
    type Target = crate::Property;
    fn run(
        &self,
        _project: Option<&ProjectConfig>,
        config: &LintConfig,
        processed: Option<&Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &crate::Property,
        _data: &LintData,
    ) -> Vec<std::sync::Arc<dyn Code>> {
        let mut codes = Vec::new();
        let Some(processed) = processed else {
            return vec![];
        };
        let Property::Entry { name, value, .. } = target else {
            return vec![];
        };
        let allow_no_extension = if let Some(toml::Value::Boolean(allow_no_extension)) = config.option("allow_no_extension") {
            *allow_no_extension
        } else {
            false
        };
        let name = name.as_str().to_lowercase();

        match value {
            Value::Array(arr) => {
                for item in &arr.items {
                    check_item(&name, item, allow_no_extension, processed, config, &mut codes);
                }
            }
            Value::Str(value) => check_str(&name, value, allow_no_extension, processed, config, &mut codes),
            _ => {}
        }
        codes
    }
}

fn check_item(
    name: &str,
    target: &crate::Item,
    allow_no_extension: bool,
    processed: &Processed,
    config: &LintConfig,
    codes: &mut Vec<Arc<dyn Code>>,
) {
    match target {
        Item::Array(values) => {
            for value in values {
                check_item(name, value, allow_no_extension, processed, config, codes);
            }
        }
        Item::Str(value) => {
            check_str(name, value, allow_no_extension, processed,  config, codes);
        }
        _ => {}
    }
}

fn check_str(name: &str, value: &Str, allow_no_extension: bool, processed: &Processed, config: &LintConfig, 
    codes: &mut Vec<Arc<dyn Code>>) {
    let value_str = value.value();
    // Skip if it contains no backslashes, probably a class name
    if !value_str.contains('\\') {
        return;
    }
    let value_str = value_str.to_lowercase();
    if value_str.starts_with("\\a3") {
        return;
    }
    if name == "sound" && value_str.starts_with("db") {
        return;
    }
    let allowed = allowed_ext(name);
    if !allowed.is_empty() {
        if value_str.is_empty() {
            return;
        }
        let ext = if value_str.contains('.') {
            value_str.split('.').next_back().unwrap_or("")
        } else {
            ""
        };
        if ext.is_empty() {
            if !allow_no_extension {
                let span = value.span().start + 1..value.span().end - 1;
                codes.push(Arc::new(CodeC11MissingExtension::new(span, processed, config.severity())));
            }
            return;
        }
        if !allowed.contains(&ext){
            let span = value.span().start + 2 + (value_str.len() - ext.len())..value.span().end - 1;
            codes.push(Arc::new(CodeC11UnusualExtension::new(span, (*allowed.first().expect("not empty extensions")).to_string(), processed, config.severity())));
        }
    }
}

fn allowed_ext(name: &str) -> Vec<&str> {
    if name.starts_with("animation") {
        if name.starts_with("animationsource") || name == "animationlist" {
            return vec![];
        }
        return vec!["rtm"];
    }
    if name.starts_with("dammage") {
        return vec!["paa"];
    }
    if name.ends_with("opticsmodel") {
        return vec!["p3d"];
    }
    if name.contains("sound") && !name.contains("soundset") {
        return vec!["wss", "ogg", "wav"];
    }
    if name.starts_with("scud") {
        return vec!["rtm"];
    }
    match name {
        "model" | "uimodel" | "modelspecial" | "modeloptics" | "modelmagazine" | "cartridge" => vec!["p3d"],
        "editorpreview" => vec!["jpg", "jpeg", "paa", "pac"],
        "uipicture" | "icon" | "picture" | "wounds" => vec!["paa", "pac"],
        _ => vec![],
    }
}

pub struct CodeC11MissingExtension {
    span: Range<usize>,
    diagnostic: Option<Diagnostic>,
    severity: Severity,
}

impl Code for CodeC11MissingExtension {
    fn ident(&self) -> &'static str {
        "L-C11ME"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/config.html#file_type")
    }

    fn message(&self) -> String {
        "a property that references a file is missing a file extension".to_string()
    }

    fn label_message(&self) -> String {
        "missing file extension".to_string()
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeC11MissingExtension {
    #[must_use]
    pub fn new(span: Range<usize>, processed: &Processed, severity: Severity) -> Self {
        Self {
            span,
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

pub struct CodeC11UnusualExtension {
    span: Range<usize>,
    expected: String,
    diagnostic: Option<Diagnostic>,
    severity: Severity,
}

impl Code for CodeC11UnusualExtension {
    fn ident(&self) -> &'static str {
        "L-C11UE"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/config.html#file_type")
    }

    fn message(&self) -> String {
        "a property that references a file has an unusual file type".to_string()
    }

    fn note(&self) -> Option<String> {
        Some(format!("expected file type {}", self.expected))
    }

    fn label_message(&self) -> String {
        "unusual file type".to_string()
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeC11UnusualExtension {
    #[must_use]
    pub fn new(span: Range<usize>, expected: String, processed: &Processed, severity: Severity) -> Self {
        Self {
            span,
            expected,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed_skip_macros(&self, self.span.clone(), processed);
        self
    }
}
