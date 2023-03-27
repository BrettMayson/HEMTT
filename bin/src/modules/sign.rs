use std::fs::{create_dir_all, File};

use git2::Repository;
use hemtt_pbo::ReadablePbo;
use hemtt_signing::BIPrivateKey;

use crate::{addons::Location, context::Context, error::Error};

use super::Module;

pub struct Sign;
impl Sign {
    pub const fn new() -> Self {
        Self
    }
}

impl Module for Sign {
    fn name(&self) -> &'static str {
        "Sign"
    }

    fn check(&self, ctx: &Context) -> Result<(), Error> {
        if ctx.config().version().git_hash().is_some() {
            Repository::open(".")?;
        }
        Ok(())
    }

    fn pre_release(&self, ctx: &Context) -> Result<(), Error> {
        let authority = get_authority(ctx, None)?;
        let addons_key = BIPrivateKey::generate(1024, &authority)?;
        create_dir_all(ctx.out_folder().join("keys"))?;
        addons_key.to_public_key().write(&mut File::create(
            ctx.out_folder()
                .join("keys")
                .join(format!("{authority}.bikey")),
        )?)?;
        ctx.addons().to_vec().iter().try_for_each(|addon| {
            let pbo_name = addon.pbo_name(&ctx.config().prefix());
            let (mut pbo, sig_location, key) = match addon.location() {
                Location::Addons => {
                    let target_pbo = {
                        let mut path = ctx.out_folder().join("addons").join(pbo_name);
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
                                .out_folder()
                                .join("optionals")
                                .join(format!("@{}", addon.pbo_name(&ctx.config().folder_name())));
                            create_dir_all(mod_root.join("keys"))?;
                            key.to_public_key().write(&mut File::create(
                                mod_root.join("keys").join(format!("{authority}.bikey")),
                            )?)?;
                            (mod_root.join("addons").join(pbo_name), key, authority)
                        } else {
                            (
                                ctx.out_folder()
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
        Ok(())
    }
}

fn get_authority(ctx: &Context, suffix: Option<&str>) -> Result<String, Error> {
    let mut authority = format!(
        "{}_{}",
        ctx.config().signing().authority().map_or_else(
            || ctx.config().prefix().to_string(),
            std::string::ToString::to_string
        ),
        ctx.config().version().get()?
    );
    if let Some(suffix) = suffix {
        authority.push_str(&format!("_{suffix}"));
    }
    Ok(authority)
}
