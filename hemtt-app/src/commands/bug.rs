use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

use crate::{Command, HEMTTError};

pub struct Bug {}

impl Command for Bug {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("bug")
            .version(*crate::VERSION)
            .about("Create a HEMTT bug report on GitHub, it will include the logs from the previous command")
    }

    fn require_project(&self) -> bool {
        false
    }

    fn run_no_project(&self, _: &clap::ArgMatches) -> Result<(), HEMTTError> {
        let project_url = String::from("URL to used project (ideally directly to used commit)");
        let project_url = if let Ok(repo) = git2::Repository::open(".") {
            if let Ok(remote) = repo.find_remote("origin") {
                if let Some(url) = remote.url() {
                    debug!("determined remote origin as `{}`", url);
                    url.to_string()
                } else {
                    debug!("tried to read remote, but it had no url");
                    project_url
                }
            } else {
                debug!("tried to use origin, but not remote was found");
                project_url
            }
        } else {
            project_url
        };
        let log_file = crate::log_path(false);
        let log = if log_file.exists() {
            format!(
                "<details>\n<summary>HEMTT Output</summary><pre>\n{}</pre></details>",
                std::fs::read_to_string(log_file)?
            )
        } else {
            String::from("**HEMTT Output:**\n\nProvide terminal output of HEMTT.")
        };
        let body = format!(
            r#"**HEMTT Version:** `{}`
**Project:** `{}`

**Description:**

Add a detailed description of the error. This makes it easier for us to fix the issue.

**Steps to reproduce:**

- Add the steps needed to reproduce the issue.

**Additional information:**

Provide any additional information that will help us solve this issue.

{}            
"#,
            *crate::VERSION,
            project_url,
            log
        );
        let url = format!(
            "https://github.com/synixebrett/HEMTT/issues/new?body={}",
            utf8_percent_encode(&body, FRAGMENT).to_string()
        );
        if webbrowser::open(&url).is_err() {
            Err(HEMTTError::Generic("Unable to open a browser".to_string()))
        } else {
            info!("Launching browser");
            Ok(())
        }
    }
}
