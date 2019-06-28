use crate::error::{HEMTTError};

#[derive(Debug, Default)]
pub struct Report {
    pub warnings: Vec<HEMTTError>,
    pub errors: Vec<HEMTTError>,
    pub can_proceed: bool,
}

impl Report {
    pub fn new() -> Self {
        Self {
            warnings: Vec::new(),
            errors: Vec::new(),
            can_proceed: true,
        }
    }

    pub fn absorb(&mut self, mut other: Self) {
        self.warnings.append(&mut other.warnings);
        self.can_proceed = self.can_proceed && other.can_proceed;
        for error in other.errors {
            self.unique_error(error);
        }
    }

    pub fn display(&self) {
        // for warning in &self.warnings {
        //     match warning {
        //         HEMTTError::GENERIC(s, v) => {
        //             warn!(s, v);
        //         },
        //         HEMTTError::LINENO(error) => {
        //             filewarn!(error);
        //         },
        //         _ => {

        //         }
        //     }
        // }
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
                    match error {
                        HEMTTError::LINENO(e) => {
                            if n == *e {
                                add = false;
                                break;
                            }
                        },
                        _ => {},
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
