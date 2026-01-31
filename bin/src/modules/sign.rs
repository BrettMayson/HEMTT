use std::sync::{Arc, RwLock};

use fs_err::File;
use hemtt_common::prefix::FILES;
use hemtt_pbo::ReadablePbo;
use hemtt_signing::{BIPrivateKey, HEMTTPrivateKey};
use hemtt_workspace::{
    addons::Location,
    reporting::{Code, Diagnostic},
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{context::Context, error::Error, report::Report};

use super::Module;

#[derive(Debug, Default)]
pub struct Sign {
    reused_key: RwLock<Option<BIPrivateKey>>,
}

impl Sign {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            reused_key: RwLock::new(None),
        }
    }
}

impl Module for Sign {
    fn name(&self) -> &'static str {
        "Sign"
    }

    fn check(&self, ctx: &Context) -> Result<Report, Error> {
        let mut report = Report::new();

        if let Some(hash) = ctx.config().signing().private_key_hash() {
            let authority = get_authority(ctx, None)?;
            let key_path = ctx
                .project_folder()
                .join(format!("{authority}.hemttprivatekey"));
            if !key_path.exists() {
                error!("Private key file `{}` does not exist", key_path.display());
                std::process::exit(1);
            }
            if crate::is_ci() {
                error!(
                    "Private key file `{}` should not be present in CI environments",
                    key_path.display()
                );
                std::process::exit(1);
            }
            let password = dialoguer::Password::new()
                .with_prompt("Enter password to decrypt private key")
                .interact()?;
            let key =
                HEMTTPrivateKey::read_encrypted(&mut fs_err::File::open(&key_path)?, &password)?;
            if key.prefix != ctx.config().prefix() {
                error!("Private key prefix does not match the project prefix");
                std::process::exit(1);
            }
            if key.project != ctx.config().name() {
                error!("Private key project does not match the project name");
                std::process::exit(1);
            }
            if key.git_hash != get_git_first_hash()? {
                error!("Private key git hash does not match the project's first commit hash");
                std::process::exit(1);
            }
            if key.validation_hash()? != hash {
                error!("Private key validation hash does not match the expected value");
                std::process::exit(1);
            }
            *self.reused_key.write().expect("rwlock poisoned") = Some(key.bi);
        }

        ctx.addons().to_vec().iter().for_each(|addon| {
            let entries = fs_err::read_dir(addon.folder())
                .expect("valid read_dir")
                .collect::<Result<Vec<_>, _>>()
                .expect("files valid");
            if entries.is_empty() {
                report.push(EmptyAddon::code(addon.folder()));
            } else if entries.len() == 1 {
                // prefix files won't end up in the PBO, so we need to ignore them
                if FILES.contains(
                    &entries[0]
                        .file_name()
                        .into_string()
                        .expect("valid file name")
                        .to_lowercase()
                        .as_str(),
                ) {
                    report.push(EmptyAddon::code(addon.folder()));
                }
            }
        });

        Ok(report)
    }

    fn pre_release(&self, ctx: &Context) -> Result<Report, Error> {
        let reused_key = self.reused_key.read().expect("rwlock poisoned");
        let (addons_key, authority) = if let Some(key) = reused_key.clone() {
            let authority = key.authority().to_string();
            (key, authority)
        } else {
            let authority = get_authority(ctx, None)?;
            (BIPrivateKey::generate(1024, &authority)?, authority)
        };
        drop(reused_key);
        fs_err::create_dir_all(
            ctx.build_folder()
                .expect("build folder exists")
                .join("keys"),
        )?;
        addons_key.to_public_key().write(&mut File::create(
            ctx.build_folder()
                .expect("build folder exists")
                .join("keys")
                .join(format!("{authority}.bikey")),
        )?)?;
        ctx.addons().to_vec().par_iter().try_for_each(|addon| {
            let pbo_name = addon.pbo_name(ctx.config().prefix());
            let (mut pbo, sig_location, key) = match addon.location() {
                Location::Addons => {
                    let target_pbo = {
                        let mut path = ctx
                            .build_folder()
                            .expect("build folder exists")
                            .join("addons")
                            .join(pbo_name);
                        path.set_extension("pbo");
                        path
                    };
                    (
                        ReadablePbo::from(File::open(&target_pbo)?)?,
                        target_pbo.with_extension(format!("pbo.{authority}.bisign")),
                        addons_key.clone(),
                    )
                }
                Location::Optionals => {
                    let (mut target_pbo, key, authority) =
                        if ctx.config().hemtt().build().optional_mod_folders() {
                            let (key, authority) = {
                                let reused_key = self.reused_key.read().expect("rwlock poisoned");
                                if let Some(key) = reused_key.clone() {
                                    let authority = key.authority().to_string();
                                    (key, authority)
                                } else {
                                    let authority = get_authority(ctx, Some(&pbo_name))?;
                                    (BIPrivateKey::generate(1024, &authority)?, authority)
                                }
                            };
                            let mod_root = ctx
                                .build_folder()
                                .expect("build folder exists")
                                .join("optionals")
                                .join(format!(
                                    "@{}",
                                    addon.pbo_name(ctx.config().hemtt().release().folder())
                                ));
                            fs_err::create_dir_all(mod_root.join("keys"))?;
                            key.to_public_key().write(&mut File::create(
                                mod_root.join("keys").join(format!("{authority}.bikey")),
                            )?)?;
                            (mod_root.join("addons").join(pbo_name), key, authority)
                        } else {
                            (
                                ctx.build_folder()
                                    .expect("build folder exists")
                                    .join(addon.location().to_string())
                                    .join(pbo_name),
                                addons_key.clone(),
                                authority.clone(),
                            )
                        };
                    target_pbo.set_extension("pbo");
                    (
                        ReadablePbo::from(File::open(&target_pbo)?)?,
                        target_pbo.with_extension(format!("pbo.{authority}.bisign")),
                        key,
                    )
                }
            };
            debug!("signing {:?}", sig_location.display());
            let sig = key.sign(&mut pbo, ctx.config().signing().version())?;
            sig.write(&mut File::create(sig_location)?)?;
            Result::<(), Error>::Ok(())
        })?;
        Ok(Report::new())
    }
}

pub fn get_authority(ctx: &Context, suffix: Option<&str>) -> Result<String, Error> {
    let config_authority = ctx.config().signing().authority().map_or_else(
        || ctx.config().prefix().to_string(),
        std::string::ToString::to_string,
    );
    let mut authority = if ctx.config().signing().private_key_hash().is_some() {
        config_authority
    } else {
        format!(
            "{}_{}",
            config_authority,
            ctx.config().version().get(ctx.workspace_path().vfs())?
        )
    };
    if let Some(suffix) = suffix {
        authority.push('_');
        authority.push_str(suffix);
    }
    Ok(authority)
}

pub fn get_git_first_hash() -> Result<String, Error> {
    let repo = git2::Repository::discover(".")?;
    // get the hash of the first commit
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    let first_commit = revwalk.last().ok_or_else(|| {
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "repository has no commits",
        ))
    })??;
    Ok(first_commit.to_string())
}

pub struct EmptyAddon {
    file: String,
}
impl Code for EmptyAddon {
    fn ident(&self) -> &'static str {
        "BSE1"
    }

    fn message(&self) -> String {
        format!("Addon `{}` has no files", self.file)
    }

    fn note(&self) -> Option<String> {
        Some("HEMTT will not be able to sign an empty PBO".to_string())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::from_code(self))
    }
}

impl EmptyAddon {
    #[must_use]
    pub fn code(file: String) -> Arc<dyn Code> {
        Arc::new(Self { file })
    }
}
