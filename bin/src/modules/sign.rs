use std::fs::{create_dir_all, File};

use git2::Repository;
use hemtt_pbo::ReadablePbo;
use hemtt_signing::BIPrivateKey;
use hemtt_workspace::addons::Location;

use crate::{context::Context, error::Error, report::Report};

use super::Module;

#[derive(Debug, Default)]
pub struct Sign;
impl Sign {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Module for Sign {
    fn name(&self) -> &'static str {
        "Sign"
    }

    fn check(&self, ctx: &Context) -> Result<Report, Error> {
        if ctx.config().version().git_hash().is_some() {
            Repository::discover(".")?;
        }
        Ok(Report::new())
    }

    fn pre_release(&self, ctx: &Context) -> Result<Report, Error> {
        let authority = get_authority(ctx, None)?;
        let addons_key = BIPrivateKey::generate(1024, &authority)?;
        create_dir_all(
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
        ctx.addons().to_vec().iter().try_for_each(|addon| {
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
                            let authority = get_authority(ctx, Some(&pbo_name))?;
                            let key = BIPrivateKey::generate(1024, &authority)?;
                            let mod_root = ctx
                                .build_folder()
                                .expect("build folder exists")
                                .join("optionals")
                                .join(format!("@{}", addon.pbo_name(&ctx.config().folder_name())));
                            create_dir_all(mod_root.join("keys"))?;
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
    let mut authority = format!(
        "{}_{}",
        ctx.config().signing().authority().map_or_else(
            || ctx.config().prefix().to_string(),
            std::string::ToString::to_string
        ),
        ctx.config().version().get(ctx.workspace_path().vfs())?
    );
    if let Some(suffix) = suffix {
        authority.push_str(&format!("_{suffix}"));
    }
    Ok(authority)
}
