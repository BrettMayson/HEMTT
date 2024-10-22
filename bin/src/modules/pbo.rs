use std::{
    fs::{create_dir_all, File},
    sync::atomic::{AtomicU16, Ordering},
};

use git2::Repository;
use hemtt_common::{
    prefix::{Prefix, FILES},
    version::Version,
};
use hemtt_pbo::WritablePbo;
use hemtt_workspace::addons::{Addon, Location};
use vfs::VfsFileType;

use crate::{context::Context, error::Error, progress::progress_bar, report::Report};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Should the optional and compat PBOs be collapsed into the addons folder
pub enum Collapse {
    /// Yes, used for development
    Yes,
    /// No, used for build and release
    No,
}

/// Builds the PBOs
///
/// # Errors
/// [`Error`] depending on the modules
/// [`Error::Io`] if the PBO fails to write
/// [`Error::Version`] if the version is invalid
/// [`Error::Git`] if the git hash is invalid
/// [`Error::Pbo`] if the PBO fails to write
pub fn build(ctx: &Context, collapse: Collapse) -> Result<Report, Error> {
    let version = ctx.config().version().get(ctx.workspace_path().vfs())?;
    let git_hash = {
        Repository::discover(".").map_or(None, |repo| {
            repo.revparse_single("HEAD").map_or(None, |rev| {
                let id = rev.id().to_string();
                Some(id)
            })
        })
    };
    let counter = AtomicU16::new(0);
    let progress = progress_bar(ctx.addons().to_vec().len() as u64).with_message("Building PBOs");
    ctx.addons()
        .to_vec()
        .iter()
        .map(|addon| {
            internal_build(ctx, addon, collapse, &version, git_hash.as_ref())?;
            progress.inc(1);
            counter.fetch_add(1, Ordering::Relaxed);
            Ok(())
        })
        .collect::<Result<Vec<_>, Error>>()?;
    progress.finish_and_clear();
    info!("Built {} PBOs", counter.load(Ordering::Relaxed));
    Ok(Report::new())
}

fn internal_build(
    ctx: &Context,
    addon: &Addon,
    collapse: Collapse,
    version: &Version,
    git_hash: Option<&String>,
) -> Result<(), Error> {
    let mut pbo = WritablePbo::new();
    let target = ctx.build_folder().expect("build folder exists");

    let pbo_name = addon.pbo_name(ctx.config().prefix());

    let target_pbo = {
        let mut path = match collapse {
            Collapse::No => match addon.location() {
                Location::Addons => target.join("addons").join(pbo_name),
                Location::Optionals => {
                    if ctx.config().hemtt().build().optional_mod_folders() {
                        target
                            .join("optionals")
                            .join(format!(
                                "@{}",
                                addon.pbo_name(ctx.config().hemtt().release().folder())
                            ))
                            .join("addons")
                            .join(pbo_name)
                    } else {
                        target.join(addon.location().to_string()).join(pbo_name)
                    }
                }
            },
            Collapse::Yes => target.join("addons").join(pbo_name),
        };
        path.set_extension("pbo");
        path
    };
    let Some(parent) = target_pbo.parent() else {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "target_pbo is at root",
        )));
    };
    if parent.exists() {
        debug!("{:?} already exists", parent);
    } else {
        debug!("creating {:?}", parent);
        create_dir_all(parent)?;
    }
    debug!(
        "building {:?} => {:?}",
        addon.folder(),
        target_pbo.display()
    );

    pbo.add_property("hemtt", env!("HEMTT_VERSION"));
    pbo.add_property("version", version.to_string());

    'entries: for entry in ctx.workspace_path().join(addon.folder())?.walk_dir()? {
        if entry.metadata()?.file_type == VfsFileType::File {
            if entry.filename() == "config.cpp" && entry.parent().join("config.bin")?.exists()? {
                continue;
            }

            if entry.filename() == "addon.toml" {
                continue;
            }

            for exclude in ctx.config().files().exclude() {
                if glob::Pattern::new(exclude)?.matches(entry.as_str()) {
                    continue 'entries;
                }
            }
            if let Some(config) = addon.config() {
                for exclude in config.files().exclude() {
                    if glob::Pattern::new(exclude)?.matches(
                        entry
                            .as_str()
                            .trim_start_matches(&format!("/{}/", addon.folder())),
                    ) {
                        continue 'entries;
                    }
                }
            }

            if FILES.contains(&entry.filename().to_lowercase().as_str()) {
                let prefix = Prefix::new(&entry.read_to_string()?)?;
                pbo.add_property("prefix", prefix.to_string());
                pbo.add_property("version", version.to_string());
                if let Some(hash) = git_hash {
                    pbo.add_property("git", hash);
                }
                continue;
            }

            let file = entry
                .as_str()
                .trim_start_matches(&format!("/{}/", addon.folder()))
                .replace('/', "\\");
            trace!("adding file {:?}", file);

            pbo.add_file(file, entry.open_file()?)?;
        }
    }
    for header in ctx.config().properties() {
        pbo.add_property(header.0, header.1.clone());
    }
    if let Some(config) = addon.config() {
        for header in config.properties() {
            pbo.add_property(header.0, header.1.clone());
        }
    }
    pbo.write(&mut File::create(target_pbo)?, true)?;
    Ok(())
}
