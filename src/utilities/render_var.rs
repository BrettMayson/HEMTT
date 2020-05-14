use crate::{Command, HEMTTError, Project};

pub struct RenderVar {}
impl Command for RenderVar {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("var")
            .about("Renders a varible, or a handlebars string")
            .arg(clap::Arg::with_name("variable").required(true))
    }

    fn can_announce(&self) -> bool {
        false
    }

    fn run(&self, args: &clap::ArgMatches, p: Project) -> Result<(), HEMTTError> {
        let mut variable = args.value_of("variable").unwrap().to_string();
        if !variable.contains("{{") {
            variable = format!("{{{{{}}}}}", variable);
        }
        println!("{}", p.render(&variable, None)?);
        Ok(())
    }
}
