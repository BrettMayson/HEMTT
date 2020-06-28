use crate::error::HEMTTError;

#[derive(Debug, Default)]
pub struct Report {
    pub errors: Vec<HEMTTError>,
    pub warnings: Vec<HEMTTError>,
    pub old: Vec<HEMTTError>,
    pub stop: Option<(bool, HEMTTError)>,
    displayed_stop: bool,
}

impl Report {
    pub const fn new() -> Self {
        Self {
            warnings: Vec::new(),
            old: Vec::new(),
            errors: Vec::new(),
            stop: None,
            displayed_stop: false,
        }
    }

    /// Absorbs another report
    pub fn absorb(&mut self, mut other: Self) {
        self.warnings.append(&mut other.warnings);
        self.old.append(&mut other.old);
        if self.stop.is_none() && other.stop.is_some() {
            self.stop = other.stop;
            self.displayed_stop = other.displayed_stop;
        };
        for error in other.errors {
            self.unique_error(error);
        }
    }

    pub fn display(&mut self) {
        for warning in &self.warnings {
            match warning {
                HEMTTError::GENERIC(s, v) => {
                    warn!("{}: {}", s, v);
                }
                HEMTTError::LINENO(error) => {
                    warn!("{:?}", error);
                }
                HEMTTError::SIMPLE(s) => {
                    warn!("{}", s);
                }
                HEMTTError::IO(s) => {
                    warn!("{}", s);
                }
                HEMTTError::PATH(s) => {
                    warn!("{}: {:#?}", &s.source, s.path);
                }
                HEMTTError::TOML(s) => {
                    warn!("{}", s);
                }
            }
        }
        self.old.append(&mut self.warnings);
        self.warnings = Vec::new();
        for error in &self.errors {
            match error {
                HEMTTError::GENERIC(s, v) => {
                    error!("{}: {}", s, v);
                }
                HEMTTError::SIMPLE(s) => {
                    error!("{}", s);
                }
                HEMTTError::LINENO(error) => {
                    error!("{:?}", error);
                }
                HEMTTError::IO(s) => {
                    error!("{}", s);
                }
                HEMTTError::PATH(s) => {
                    error!("{}: {:#?}", &s.source, s.path);
                }
                HEMTTError::TOML(s) => {
                    error!("{}", s);
                }
            }
        }
        if !self.displayed_stop && self.stop.is_some() {
            self.displayed_stop = true;
            let (fatal, error) = self.stop.as_ref().unwrap();
            if *fatal {
                match error {
                    HEMTTError::GENERIC(s, v) => {
                        error!("{}: {}", s, v);
                    }
                    HEMTTError::SIMPLE(s) => {
                        error!("{}", s);
                    }
                    HEMTTError::LINENO(error) => {
                        error!("{:?}", error);
                    }
                    HEMTTError::IO(s) => {
                        error!("{}", s);
                    }
                    HEMTTError::PATH(s) => {
                        error!("{}: {:#?}", &s.source, s.path);
                    }
                    HEMTTError::TOML(s) => {
                        error!("{}", s);
                    }
                }
            }
        }
    }

    /// Adds an error if it does not exist in the report
    pub fn unique_error(&mut self, error: HEMTTError) {
        match error {
            HEMTTError::LINENO(n) => {
                let lineerror = n.clone();
                let mut add = true;
                for error in &self.errors {
                    if let HEMTTError::LINENO(e) = error {
                        if n == *e {
                            add = false;
                            break;
                        }
                    }
                }
                if add {
                    self.errors.push(HEMTTError::LINENO(lineerror));
                }
            }
            _ => {
                self.errors.push(error);
            }
        }
    }
}
