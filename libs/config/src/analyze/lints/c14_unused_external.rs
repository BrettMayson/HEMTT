use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc, sync::Arc};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{analyze::LintData, Class, Config, Property};

crate::analyze::lint!(LintC14UnusedExternal);

impl Lint<LintData> for LintC14UnusedExternal {
    fn ident(&self) -> &'static str {
        "unused_external"
    }

    fn sort(&self) -> u32 {
        140
    }

    fn description(&self) -> &'static str {
        "Reports on external classes that are never used in the config"
    }

    fn documentation(&self) -> &'static str {
        "### Example

**Incorrect**
```hpp
class x;
```

**Correct**
```hpp
```
"
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::help() // todo pedantic
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner<LintData> for Runner {
    type Target = Config;
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
            used: true,
            upper: None,
            subclasses: HashMap::new(),
        }));
        check(&target.0, &root);
        // println!("root: {root:?}");
        Cfg::check_unused(&root, processed)
    }
}

struct Cfg {
    class: Class,
    used: bool,
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
                    .map_or_else(|| "None".to_string(), |c| c.value.clone())
            },
        );
        write!(
            f,
            "Cfg {{ name: {}, used: {}, upper: {}",
            self.class, self.used, upper_name
        )?;
        for subclass in &self.subclasses {
            write!(f, "\n-{:?}", subclass.1.borrow())?;
        }
        write!(f, " }}")
    }
}

impl Cfg {
    #[must_use]
    fn check_unused(cfg: &Rc<RefCell<Self>>, processed: &Processed) -> Codes {
        let mut codes: Codes = Vec::new();
        if !cfg.borrow().used {
            codes.push(Arc::new(CodeC14UnusedExternal::new(
                cfg.borrow().class.clone(),
                processed,
            )));
        }
        for subclass in cfg.borrow().subclasses.values() {
            codes.extend(Self::check_unused(subclass, processed));
        }
        codes
    }
    #[must_use]
    fn insert_class(cfg: &Rc<RefCell<Self>>, class: &Class, external: bool) -> Rc<RefCell<Self>> {
        let name = class
            .name()
            .expect("class has a name")
            .value
            .to_ascii_lowercase();
        let new_class = Rc::new(RefCell::new(Self {
            class: class.clone(),
            used: !external,
            upper: Some(cfg.clone()),
            subclasses: HashMap::new(),
        }));
        cfg.borrow_mut().subclasses.insert(name, new_class.clone());
        new_class
    }
    #[must_use]
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
            let mut parent_cfg = result_parent.borrow_mut();
            parent_cfg.used = true;
            new_class = Some(Rc::new(RefCell::new(Self {
                class: class.clone(),
                used: true,
                upper: Some(cfg.clone()),
                subclasses: parent_cfg.subclasses.clone(),
            })));
            break;
        }
        let found = new_class.is_some();
        if !found {
            new_class = Some(Rc::new(RefCell::new(Self {
                class: class.clone(),
                used: true,
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

fn check(properties: &[Property], base: &Rc<RefCell<Cfg>>) {
    for property in properties {
        if let Property::Class(c) = property {
            match c {
                Class::Root { properties } => {
                    check(properties, &base.clone());
                }
                Class::External { .. } => {
                    let _class = Cfg::insert_class(base, c, true);
                }
                Class::Local {
                    name: _,
                    parent,
                    properties,
                    err_missing_braces: _,
                } => {
                    let new_class = if parent.is_none() {
                        Cfg::insert_class(base, c, false)
                    } else {
                        let (class, _found) = Cfg::insert_inherited(
                            base,
                            c,
                            &parent.clone().expect("parent exists").value,
                        );
                        class
                    };
                    check(properties, &new_class);
                }
            }
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeC14UnusedExternal {
    class: Class,
    class_name: String,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeC14UnusedExternal {
    fn ident(&self) -> &'static str {
        "L-C14"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/config.html#unused_external")
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn message(&self) -> String {
        "external class is never used".to_string()
    }

    fn label_message(&self) -> String {
        format!("{} is never used", self.class_name)
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeC14UnusedExternal {
    #[must_use]
    pub fn new(class: Class, processed: &Processed) -> Self {
        Self {
            class,
            class_name: String::new(),
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let Some(name) = self.class.name() else {
            panic!("CodeC14UnusedExternal::generate_processed called on class without name");
        };
        self.class_name = name.value.clone();
        self.diagnostic = Diagnostic::from_code_processed(&self, name.span.clone(), processed);
        self
    }
}
