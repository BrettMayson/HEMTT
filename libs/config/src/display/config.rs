use crate::Config;

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for property in &self.0 {
            write!(f, "{property}")?;
        }
        Ok(())
    }
}
