use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use regex::{Captures, Regex};
use rlua::Lua;
use walkdir::WalkDir;

use crate::{Command, HEMTTError};

mod version;

#[derive(Default)]
pub struct Template {}

impl Template {
    pub const fn new() -> Self {
        Self {}
    }

    #[allow(dead_code)]
    fn eval_file<F: FnOnce(rlua::Context)>(&self, file: &str, setup: F) -> String {
        let lua = Lua::new();
        lua.context(|lua_ctx| {
            self.setup_lua(lua_ctx);
            setup(lua_ctx);
            lua_ctx
                .load(&std::fs::read_to_string(file).unwrap())
                .eval::<String>()
                .unwrap()
        })
    }

    fn eval_file_empty<F: FnOnce(rlua::Context)>(&self, file: &str, setup: F) {
        let lua = Lua::new();
        lua.context(|lua_ctx| {
            self.setup_lua(lua_ctx);
            setup(lua_ctx);
            lua_ctx.load(&std::fs::read_to_string(file).unwrap()).eval::<()>().unwrap();
        })
    }

    fn setup_lua(&self, lua_ctx: rlua::Context) {
        let globals = lua_ctx.globals();

        let lua_print = lua_ctx
            .create_function(|_, text: String| {
                println!("{}", text);
                Ok(())
            })
            .unwrap();
        globals.set("print", lua_print).unwrap();

        let lua_read_file = lua_ctx
            .create_function(|_, file: String| Ok(std::fs::read_to_string(file).unwrap()))
            .unwrap();
        globals.set("read_file", lua_read_file).unwrap();

        let lua_copy = lua_ctx
            .create_function(|ctx, (src, dst): (String, String)| {
                let src_path = Path::new(&src);
                if !src_path.exists() {
                    return Ok(());
                }
                if src_path.is_dir() {
                    let dst_path = PathBuf::from(&dst);
                    let mut ancestors = dst_path.ancestors();
                    ancestors.next();
                    if let Some(ancestor) = ancestors.next() {
                        create_dir!(ancestor).unwrap();
                    }
                    let mut options = fs_extra::dir::CopyOptions::new();
                    options.copy_inside = true;
                    options.overwrite = true;
                    fs_extra::dir::copy(src, &dst, &options).unwrap();
                    let re = Regex::new(r"(?m)%%([A-Za-z]+)%%").unwrap();
                    for entry in WalkDir::new(dst) {
                        let path = entry.unwrap();
                        if !path.path().is_file() {
                            continue;
                        }
                        let mut variables: HashMap<&str, String> = HashMap::new();
                        // TODO replace a with type ascription
                        let a: Result<String, _> = ctx.globals().get("new_addon");
                        if let Ok(v) = a {
                            variables.insert("addon", v.clone());
                            variables.insert("ADDON", v.to_uppercase());
                        }
                        let contents = std::fs::read_to_string(path.path()).unwrap();
                        let result = re.replace_all(&contents, |caps: &Captures| {
                            let dft = String::from(&caps[1]);
                            variables.get(&caps[1]).unwrap_or(&dft).to_string()
                        });
                        let mut out = create_file!(path.path()).unwrap();
                        out.write_all(result.into_owned().as_bytes()).unwrap();
                    }
                } else {
                    copy_file!(src, dst).unwrap();
                }
                Ok(())
            })
            .unwrap();
        globals.set("fs_copy", lua_copy).unwrap();
    }

    fn run_script<F: FnOnce(rlua::Context)>(&self, file: &str, setup: F) {
        if Path::new(&script(file)).exists() {
            self.eval_file_empty(&script(file), setup);
        } else {
            println!(
                "No {} script exists for this template, report this to the template creator.",
                file
            );
        }
    }
}

impl Command for Template {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("template")
            .version(*crate::VERSION)
            .about("Manage the project's template")
            .subcommand(clap::SubCommand::with_name("init").about("Initialize the template"))
            .subcommand(
                clap::SubCommand::with_name("addon").about("Create a new addon").arg(
                    clap::Arg::with_name("name")
                        .help("Name of the addon to create")
                        .required(true),
                ),
            )
            .subcommand(
                clap::SubCommand::with_name("function")
                    .arg(clap::Arg::with_name("addon").help("Addon to add function to").required(true))
                    .arg(clap::Arg::with_name("name").help("Name of the function").required(true)),
            )
    }

    fn run(&self, a: &clap::ArgMatches, p: crate::project::Project) -> Result<(), HEMTTError> {
        if p.template.is_empty() {
            return Ok(());
        }
        match a.subcommand() {
            ("addon", Some(args)) => {
                let name = args.value_of("name").unwrap();
                if Path::new(&format!("addons/{}", name)).exists() {
                    println!("addons/{} already exists", name);
                    return Ok(());
                }
                self.run_script("addon", |lua_ctx| {
                    let globals = lua_ctx.globals();
                    globals.set("new_addon", name).unwrap();
                });
            }
            ("function", Some(args)) => {
                let addon = args.value_of("addon").unwrap();
                let name = args.value_of("name").unwrap();
                self.run_script("function_add", |lua_ctx| {
                    let globals = lua_ctx.globals();
                    globals.set("addon", addon).unwrap();
                    globals.set("name", name).unwrap();
                });
            }
            ("init", _) => {
                self.run_script("init", |_| {});
            }
            _ => println!("Not implemented"),
        }
        Ok(())
    }
}

pub fn script(name: &str) -> String {
    format!("./.hemtt/template/scripts/{}.lua", name)
}
