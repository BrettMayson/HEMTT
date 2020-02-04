use webbrowser;

use crate::{Command, HEMTTError};

pub struct Bug {}

impl Command for Bug {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("bug")
            .version(*crate::VERSION)
            .about("Create a HEMTT bug report on GitHub")
    }

    fn require_project(&self) -> bool {
        false
    }

    fn run_no_project(&self, _: &clap::ArgMatches) -> Result<(), HEMTTError> {
        let url = format!("https://github.com/synixebrett/HEMTT/issues/new?body=%2A%2AHEMTT%20Version%3A%2A%2A%20%60{}%60%0A%2A%2AProject%3A%2A%2A%20URL%20to%20used%20project%20%28ideally%20directly%20to%20used%20commit%29%0A%0A%2A%2ADescription%3A%2A%2A%0A%0AAdd%20a%20detailed%20description%20of%20the%20error.%20This%20makes%20it%20easier%20for%20us%20to%20fix%20the%20issue.%0A%0A%2A%2ASteps%20to%20reproduce%3A%2A%2A%0A%0A-%20Add%20the%20steps%20needed%20to%20reproduce%20the%20issue.%0A%0A%2A%2AAdditional%20information%3A%2A%2A%0A%0AProvide%20any%20additional%20information%20that%20will%20help%20us%20solve%20this%20issue.%0A%0A%2A%2AHEMTT%20Output%3A%2A%2A%0A%0AProvide%20terminal%20output%20of%20HEMTT.%0A",
            *crate::VERSION
        );
        if webbrowser::open(&url).is_err() {
            Err(HEMTTError::SIMPLE("Unable to open a browser".to_string()))
        } else {
            Ok(())
        }
    }
}
