use git2::Repository;

use crate::{Error, context::Context, modules::Module, report::Report};

#[derive(Default)]
pub struct Git;
impl Module for Git {
    fn name(&self) -> &'static str {
        "git"
    }

    fn check(&self, ctx: &Context) -> Result<Report, Error> {
        let mut using_private_key = false;
        if let Ok(repo) = Repository::discover(".") {
            for entry in walkdir::WalkDir::new(ctx.project_folder()) {
                let entry = entry?;
                let name = entry.file_name().to_string_lossy();
                if name.ends_with(".hemttprivatekey") || name.ends_with(".biprivatekey") {
                    using_private_key = true;
                    let rel_path = entry
                        .path()
                        .strip_prefix(ctx.project_folder())
                        .unwrap_or_else(|_| entry.path());
                    let potential_leak = {
                        let mut revwalk = repo.revwalk()?;
                        revwalk.push_head()?;
                        revwalk.any(|oid_result| {
                            if let Ok(oid) = oid_result
                                && let Ok(commit) = repo.find_commit(oid)
                                && let Ok(tree) = commit.tree()
                            {
                                return tree.get_name(&rel_path.display().to_string()).is_some();
                            }
                            false
                        })
                    };
                    let status = repo.status_file(rel_path);
                    if !potential_leak
                        && (status == Ok(git2::Status::IGNORED)
                            || status == Ok(git2::Status::WT_NEW))
                    {
                        continue;
                    }
                    error!("==== STOP ====");
                    error!("You are tracking a private key in git: {}", name);
                    error!(
                        "This is a serious security risk, as anyone with access to the repository will be able to sign PBOs as you."
                    );
                    if potential_leak {
                        error!("The key has been committed to the repository history.");
                        println!();
                        error!("You must consider the key compromised, and generate a new one.");
                        println!();
                    } else {
                        error!(
                            "The key is only present in the working directory, and has not been committed yet."
                        );
                    }
                    error!("HEMTT will refuse to build until this is resolved.");
                    error!("==== STOP ====");
                    std::process::exit(1);
                }
            }
        } else if ctx.config().version().git_hash().is_some() {
            return Err(Error::NotInGitRepository(
                "Version is set to use git hash".to_string(),
            ));
        }

        let gitignore = ctx.project_folder().join(".gitignore");
        if using_private_key && gitignore.exists() {
            let mut search = false;
            let content = fs_err::read_to_string(&gitignore)?;
            if !content.contains(".hemttprivatekey") {
                warn!(".hemttprivatekey is not in your .gitignore file.");
                search = true;
            }
            if !content.contains(".biprivatekey") {
                warn!(".biprivatekey is not in your .gitignore file.");
                search = true;
            }
            if search {
                // Check for any file with a .hemttprivatekey or .biprivatekey extension in the current directory and subdirectories
                for entry in walkdir::WalkDir::new(ctx.project_folder()) {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file()
                        && let Some(ext) = path.extension()
                        && (ext == "hemttprivatekey" || ext == "biprivatekey")
                    {
                        error!("==== STOP ====");
                        error!(
                            "You have private key files in your project directory: {}",
                            path.display()
                        );
                        error!(
                            "These files are not ignored by git, which is a serious security risk."
                        );
                        error!("HEMTT will refuse to build until this is resolved.");
                        error!("==== STOP ====");
                        std::process::exit(1);
                    }
                }
            }
        }

        Ok(Report::new())
    }
}
