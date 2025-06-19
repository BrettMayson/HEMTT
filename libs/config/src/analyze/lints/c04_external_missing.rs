use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc, sync::Arc};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed},
};

use crate::{analyze::LintData, Class, Config, Property};

crate::analyze::lint!(LintC04ExternalMissing);

impl Lint<LintData> for LintC04ExternalMissing {
    fn ident(&self) -> &'static str {
        "external_missing"
    }

    fn sort(&self) -> u32 {
        40
    }

    fn description(&self) -> &'static str {
        "Reports on classes that extend an external class that is not present in the config"
    }

    fn documentation(&self) -> &'static str {
        "### Example

**Incorrect**
```hpp
class MyClass: ExternalClass {
    value = 1;
};
```

**Correct**
```hpp
class ExternalClass;
class MyClass: ExternalClass {
    value = 1;
};
```

### Explanation

Classes that extend an external class must be declared in the config.

Read more about [class inheritance](https://community.bistudio.com/wiki/Class_Inheritance).
"
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::fatal()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner<LintData> for Runner {
    type Target = Config;
    #[allow(clippy::let_and_return)]
    fn run(
        &self,
        _project: Option<&ProjectConfig>,
        _config: &LintConfig,
        processed: Option<&Processed>,
        target: &Config,
        _data: &LintData,
    ) -> Vec<std::sync::Arc<dyn Code>> {
        let Some(processed) = processed else {
            return vec![];
        };
        let root = Rc::new(RefCell::new(Cfg {
            class: Class::Root { properties: vec![] },
            upper: None,
            subclasses: HashMap::new(),
        }));
        let codes = check(&target.0, &root, processed);
        // println!("root: {root:?}");
        codes
    }
}

struct Cfg {
    class: Class,
    upper: Option<Rc<RefCell<Cfg>>>,
    subclasses: HashMap<String, Rc<RefCell<Cfg>>>,
}
impl fmt::Debug for Cfg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let upper_name = self.upper.as_ref().map_or_else(
            || "None".to_string(),
            |p| {
                p.borrow()
                    .class
                    .name()
                    .expect("class has a name")
                    .value
                    .clone()
            },
        );
        write!(f, "Cfg {{ name: {}, upper: {}", self.class, upper_name)?;
        for subclass in &self.subclasses {
            write!(f, "\n-{:?}", subclass.1.borrow())?;
        }
        write!(f, " }}")
    }
}

impl Cfg {
    fn insert_class(cfg: &Rc<RefCell<Self>>, class: &Class) -> Rc<RefCell<Self>> {
        let name = class
            .name()
            .expect("class has a name")
            .value
            .to_ascii_lowercase();
        let new_class = Rc::new(RefCell::new(Self {
            class: class.clone(),
            upper: Some(cfg.clone()),
            subclasses: HashMap::new(),
        }));
        cfg.borrow_mut().subclasses.insert(name, new_class.clone());
        new_class
    }
    #[allow(clippy::assigning_clones)]
    fn insert_inherited(
        cfg: &Rc<RefCell<Self>>,
        class: &Class,
        parent: &str,
    ) -> (Rc<RefCell<Self>>, bool) {
        let name = class
            .name()
            .expect("class has a name")
            .value
            .to_ascii_lowercase();
        let mut new_class = None;
        let mut search_tier = Some(cfg.clone());
        while let Some(search) = search_tier {
            let parent_key = parent.to_ascii_lowercase();
            let result_hash = search.borrow().subclasses.get(&parent_key).cloned();
            let Some(result_parent) = result_hash else {
                search_tier = search.borrow().upper.clone();
                continue;
            };
            let parent_cfg = result_parent.borrow_mut();
            new_class = Some(Rc::new(RefCell::new(Self {
                class: class.clone(),
                upper: Some(cfg.clone()),
                subclasses: parent_cfg.subclasses.clone(),
            })));
            break;
        }
        let found = new_class.is_some();
        if !found {
            new_class = Some(Rc::new(RefCell::new(Self {
                class: class.clone(),
                upper: Some(cfg.clone()),
                subclasses: HashMap::new(),
            })));
        }
        cfg.borrow_mut()
            .subclasses
            .insert(name, new_class.clone().expect("new_class exists"));
        (new_class.expect("new_class exists"), found)
    }
}

fn check(properties: &[Property], base: &Rc<RefCell<Cfg>>, processed: &Processed) -> Codes {
    let mut codes: Codes = Vec::new();
    for property in properties {
        if let Property::Class(c) = property {
            match c {
                Class::Root { properties } => {
                    codes.extend(check(properties, &base.clone(), processed));
                }
                Class::External { .. } => {
                    let _class = Cfg::insert_class(base, c);
                }
                Class::Local {
                    name: _,
                    parent,
                    properties,
                    err_missing_braces: _,
                } => {
                    let new_class = if parent.is_none() {
                        Cfg::insert_class(base, c)
                    } else {
                        let (class, found) = Cfg::insert_inherited(
                            base,
                            c,
                            &parent.clone().expect("parent exists").value,
                        );
                        if !found {
                            codes.push(Arc::new(CodeC04ExternalMissing::new(c.clone(), processed)));
                        }
                        class
                    };
                    codes.extend(check(properties, &new_class, processed));
                }
            }
        }
    }

    codes
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeC04ExternalMissing {
    class: Class,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeC04ExternalMissing {
    fn ident(&self) -> &'static str {
        "L-C04"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/config.html#external_missing")
    }

    fn message(&self) -> String {
        "class's parent is not present".to_string()
    }

    fn label_message(&self) -> String {
        "not present in config".to_string()
    }

    fn help(&self) -> Option<String> {
        self.class.parent().map(|parent| {
            format!(
                "add `class {};` to the config to declare it as external",
                parent.as_str(),
            )
        })
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeC04ExternalMissing {
    #[must_use]
    pub fn new(class: Class, processed: &Processed) -> Self {
        Self {
            class,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let Some(parent) = self.class.parent() else {
            panic!("CodeC04ExternalMissing::generate_processed called on class without parent");
        };
        self.diagnostic = Diagnostic::from_code_processed(&self, parent.span.clone(), processed);
        self
    }
}
