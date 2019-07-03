use crate::error::{HEMTTError};

#[derive(Debug, Default)]
pub struct Report {
    pub warnings: Vec<HEMTTError>,
    pub errors: Vec<HEMTTError>,
    pub stop: Option<HEMTTError>,
}

impl Report {
    pub fn new() -> Self {
        Self {
            warnings: Vec::new(),
            errors: Vec::new(),
            stop: None,
        }
    }

    pub fn absorb(&mut self, mut other: Self) {
        self.warnings.append(&mut other.warnings);
        if self.stop.is_none() && other.stop.is_some() {
            self.stop = other.stop;
        };
        for error in other.errors {
            self.unique_error(error);
        }
    }

    pub fn display(&self) {
        for warning in &self.warnings {
            match warning {
                HEMTTError::GENERIC(s, v) => {
                    warnmessage!(s, v);
                },
                HEMTTError::LINENO(error) => {
                    filewarn!(error);
                },
                HEMTTError::SIMPLE(s) => {
                    warn!(s);
                },
                _ => {

                }
            }
        }
        for error in &self.errors {
            match error {
                HEMTTError::GENERIC(s, v) => {
                    errormessage!(s, v);
                },
                HEMTTError::SIMPLE(s) => {
                    error!(s);
                },
                HEMTTError::LINENO(error) => {
                    fileerror!(error);
                },
                _ => {

                }
            }
        }
    }

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
            },
            _ => {
                self.errors.push(error);
            }
        }
    }
}
