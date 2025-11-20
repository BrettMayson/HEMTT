use std::{
    cell::RefCell, io::Write, path::Path, rc::Rc, sync::{atomic::AtomicU16, Arc, Once, OnceLock}
};

use hemtt_common::config::{LintConfig, ProjectConfig, RuntimeArguments};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};
use indexmap::IndexMap;

use crate::{analyze::LintData, Class, Config, Property};

const PATH: &str = ".hemttout/unused_external_classes.txt";

crate::analyze::lint!(LintC14UnusedExternal);

impl Lint<LintData> for LintC14UnusedExternal {
    fn ident(&self) -> &'static str {
        "unused_external"
    }
    fn sort(&self) -> u32 {
        140
    }
    fn description(&self) -> &'static str {
        "Reports on external classes that are never used"
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
        LintConfig::warning()
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
        config: &LintConfig,
        processed: Option<&Processed>,
        runtime: &hemtt_common::config::RuntimeArguments,
        target: &Config,
        _data: &LintData,
    ) -> Vec<std::sync::Arc<dyn Code>> {
        static CLEANUP_PATH: Once = Once::new();
        CLEANUP_PATH.call_once(|| {
            if Path::new(".hemttout").exists() {
                let _ = fs_err::remove_file(PATH);
                let _ = fs_err::File::create(PATH);
            }
        });
        let Some(processed) = processed else {
            return vec![];
        };
        let root = Rc::new(RefCell::new(ClassNode {
            class: Class::Root { properties: vec![] },
            used: true,
            upper: None,
            subclasses: IndexMap::new(),
        }));
        check(&target.0, &root);
        let mut file = if Path::new(PATH).exists() {
            Some(match fs_err::OpenOptions::new()
                .append(true)
                .create(true)
                .open(PATH)
            {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Failed to open {PATH}: {e}");
                    return vec![];
                }
            })
        } else {
            None
        };
        ClassNode::check_unused(&root, &mut Vec::new(), processed, config, &mut file, runtime)
    }
}
struct ClassNode {
    class: Class,
    used: bool,
    upper: Option<Rc<RefCell<Self>>>,
    subclasses: IndexMap<String, Rc<RefCell<Self>>>, // keep insertion order constant
}

impl ClassNode {
    #[must_use]
    fn check_unused(
        cfg: &Rc<RefCell<Self>>,
        reported: &mut Vec<Class>,
        processed: &Processed,
        config: &LintConfig,
        file: &mut Option<fs_err::File>,
        runtime: &hemtt_common::config::RuntimeArguments,
    ) -> Codes {
        let mut codes: Codes = Vec::new();
        if !cfg.borrow().used && !reported.contains(&cfg.borrow().class) {
            reported.push(cfg.borrow().class.clone());
            codes.push(Arc::new(CodeC14UnusedExternal::new(
                cfg.borrow().class.clone(),
                processed,
                config.severity(),
                runtime,
            )));
            let name = cfg.borrow().class.name().expect("class has a name").clone();
            let pos = processed
                .mapping(name.span.start)
                .expect("start position exists")
                .original();
            if let Some(file) = file {
                writeln!(
                    file,
                    "{} - {}:{}:{}",
                    name.as_str(),
                    pos.path().as_str().trim_start_matches('/'),
                    pos.start().1 .0,
                    pos.start().1 .1 + 1,
                )
                .expect("Failed to write to file");
            }
        }
        for subclass in cfg.borrow().subclasses.values() {
            let inner_codes = Self::check_unused(subclass, reported, processed, config, file, runtime);
            codes.extend(inner_codes);
        }
        codes
    }

    #[must_use]
    fn get_standalone_class(
        cfg: &Rc<RefCell<Self>>,
        class: &Class,
        external: bool,
    ) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            class: class.clone(),
            used: !external,
            upper: Some(cfg.clone()),
            subclasses: IndexMap::new(),
        }))
    }

    #[must_use]
    #[allow(clippy::assigning_clones)]
    fn get_inherited_class(
        cfg: &Rc<RefCell<Self>>,
        class: &Class,
        parent: &str,
    ) -> (Rc<RefCell<Self>>, bool) {
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
                subclasses: IndexMap::new(),
            })));
        }
        (new_class.expect("new_class exists"), found)
    }
}

fn check(properties: &[Property], base: &Rc<RefCell<ClassNode>>) {
    for property in properties {
        if let Property::Class(c) = property {
            let name = c
                .name()
                .map_or_else(|| "None".to_string(), |name| name.value.clone())
                .to_ascii_lowercase();
            match c {
                Class::Root { properties } => {
                    check(properties, base);
                }
                Class::External { .. } => {
                    let new_class = ClassNode::get_standalone_class(base, c, true);
                    base.borrow_mut().subclasses.insert(name, new_class);
                }
                Class::Local {
                    name: _,
                    parent,
                    properties,
                    err_missing_braces: _,
                } => {
                    let new_class = if parent.is_none() {
                        ClassNode::get_standalone_class(base, c, false)
                    } else {
                        let (class, _found) = ClassNode::get_inherited_class(
                            base,
                            c,
                            &parent.clone().expect("parent exists").value,
                        );
                        class
                    };
                    check(properties, &new_class);
                    base.borrow_mut().subclasses.insert(name, new_class);
                }
            }
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeC14UnusedExternal {
    severity: Severity,
    class: Class,
    class_name: String,
    diagnostic: Option<Diagnostic>,
    count: Arc<AtomicU16>,
    first: bool,
    explicit: bool,
}

impl Code for CodeC14UnusedExternal {
    fn ident(&self) -> &'static str {
        "L-C14"
    }
    fn link(&self) -> Option<&str> {
        Some("/lints/config.html#unused_external")
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn message(&self) -> String {
        format!("external class {} is never used", self.class_name)
    }
    fn label_message(&self) -> String {
        "never used".to_string()
    }
    fn help(&self) -> Option<String> {
        None
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        if self.explicit {
            return self.diagnostic.clone();
        }
        let count = self.count.load(std::sync::atomic::Ordering::Relaxed);
        if count <= 5 {
            self.diagnostic.clone()
        } else if self.first {
            Some(
                Diagnostic::from_code(self)
                    .set_message(format!("There are {count} unused external classes")),
            )
        } else {
            None
        }
    }
    fn note(&self) -> Option<String> {
        if self.explicit {
            return None;
        }
        let count = self.count.load(std::sync::atomic::Ordering::Relaxed);
        if count > 5 && self.first {
            Some(format!(
                "A list has been generated in {PATH}, use `hemtt check -Lc14` to output warnings",
            ))
        } else {
            None
        }
    }
}

impl CodeC14UnusedExternal {
    #[must_use]
    pub fn new(
        class: Class,
        processed: &Processed,
        severity: Severity,
        runtime: &RuntimeArguments,
    ) -> Self {
        static COUNT: OnceLock<Arc<AtomicU16>> = OnceLock::new();
        let count = COUNT.get_or_init(|| Arc::new(AtomicU16::new(0))).clone();
        let first = count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) == 0;
        Self {
            severity,
            class,
            class_name: String::new(),
            diagnostic: None,
            count,
            first,
            explicit: runtime
                .explicit_lints()
                .iter()
                .any(|l| l.eq_ignore_ascii_case("c14")),
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
