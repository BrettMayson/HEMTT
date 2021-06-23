use hemtt::{Addon, HEMTTError};
use hemtt_handlebars::Variables;
use vfs::VfsPath;

use super::Context;

pub struct AddonContext<'a, 'b> {
    global: &'b Context<'a>,
    addon: Addon,
    fs: VfsPath,
    prefix: String,

    failed: Option<HEMTTError>,
    skip: bool,
}

impl<'a, 'b> AddonContext<'a, 'b> {
    pub fn new(global: &'b Context<'a>, addon: Addon) -> Result<Self, HEMTTError> {
        let fs = global.vfs().join(addon.source())?;
        let prefix_file = fs.join("$PBOPREFIX$")?;
        let prefix_gen = format!(
            "{}\\{}\\{}",
            global.project.mainprefix(),
            global.project.prefix(),
            addon.source()
        )
        .replace("/", "\\");
        let prefix = if prefix_file.exists()? {
            let mut source = String::new();
            prefix_file.open_file()?.read_to_string(&mut source)?;
            let mut prefix = "";
            'search: for line in source.lines() {
                if line.is_empty() {
                    break;
                }

                let eq: Vec<String> = line.split('=').map(|s| s.to_string()).collect();
                if eq.len() == 1 {
                    prefix = line.trim_matches('\\');
                    break 'search;
                } else {
                    let header = eq[0].clone();
                    if header == "prefix" {
                        prefix = line.trim_matches('\\');
                        break 'search;
                    }
                }
            }
            let prefix =
                hemtt_handlebars::render(prefix, &Variables::from(global.project())).unwrap();
            if prefix.is_empty() {
                warn!("Could not determine a prefix for {} using the $PBOPREFIX$ file, a prefix will be generated", addon.source());
                prefix_gen
            } else {
                debug!("Using prefix from $PBOPREFIX$ for {}", addon.source());
                prefix.to_string()
            }
        } else {
            debug!("Using generated prefix for {}", addon.source());
            prefix_gen
        };
        Ok(Self {
            global,
            addon,
            fs,
            prefix,

            failed: None,
            skip: false,
        })
    }

    pub fn global(&self) -> &Context {
        &self.global
    }

    pub fn failed(&self) -> bool {
        self.failed.is_some()
    }

    pub fn get_failed(&self) -> &Option<HEMTTError> {
        &self.failed
    }

    pub fn set_failed(&mut self, err: HEMTTError) {
        self.failed = Some(err);
    }

    pub fn skip(&self) -> bool {
        self.skip
    }

    pub fn set_skip(&mut self, skip: bool) {
        self.skip = skip;
    }

    pub fn fs(&self) -> &VfsPath {
        &self.fs
    }

    pub fn addon(&self) -> &Addon {
        &self.addon
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn info(&self, message: &str) {
        let (stage, task) = self.global.message_info.read().unwrap().clone();
        info!(
            "[{}] [{:^width$}] [{}] {}",
            stage,
            task,
            self.addon.name(),
            message,
            width = self.global.task_pad()
        )
    }

    pub fn warn(&self, message: &str) {
        let (stage, task) = self.global.message_info.read().unwrap().clone();
        warn!(
            "[{}] [{:^width$}] [{}] {}",
            stage,
            task,
            self.addon.name(),
            message,
            width = self.global.task_pad()
        )
    }

    pub fn error(&self, message: &str) {
        let (stage, task) = self.global.message_info.read().unwrap().clone();
        error!(
            "[{}] [{:^width$}] [{}] {}",
            stage,
            task,
            self.addon.name(),
            message,
            width = self.global.task_pad()
        )
    }

    pub fn debug(&self, message: &str) {
        let (stage, task) = self.global.message_info.read().unwrap().clone();
        debug!(
            "[{}] [{:^width$}] [{}] {}",
            stage,
            task,
            self.addon.name(),
            message,
            width = self.global.task_pad()
        )
    }

    pub fn trace(&self, message: &str) {
        let (stage, task) = self.global.message_info.read().unwrap().clone();
        trace!(
            "[{}] [{:^width$}] [{}] {}",
            stage,
            task,
            self.addon.name(),
            message,
            width = self.global.task_pad()
        )
    }
}

pub struct AddonListContext<'a, 'b> {
    global: &'b Context<'a>,
    addons: Vec<AddonContext<'a, 'b>>,
}

impl<'a, 'b> AddonListContext<'a, 'b> {
    pub fn new(global: &'b Context<'a>, addons: Vec<Addon>) -> Result<Self, HEMTTError> {
        Ok(Self {
            global,
            addons: addons
                .into_iter()
                .map(|addon| AddonContext::new(global, addon))
                .collect::<Result<Vec<AddonContext<'a, 'b>>, HEMTTError>>()?,
        })
    }

    pub fn global(&self) -> &Context {
        &self.global
    }

    pub fn addons(&self) -> &Vec<AddonContext<'a, 'b>> {
        &self.addons
    }

    pub fn mut_addons(&mut self) -> &mut Vec<AddonContext<'a, 'b>> {
        &mut self.addons
    }

    pub fn failed(&self) -> bool {
        self.addons().iter().any(|a| a.failed())
    }
}
