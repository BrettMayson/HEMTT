use std::{
    fs::File,
    path::{Path, PathBuf},
};

use hemtt_pbo::{prefix::FILES, Prefix, WritablePbo};
use vfs::VfsFileType;

use crate::{addons::Location, context::Context, error::Error};

pub fn dev(ctx: &Context) -> Result<(), Error> {
    get_pbos(ctx, &PathBuf::from(Location::Addons.to_string()))
}

pub fn release(_ctx: &Context) -> Result<(), Error> {
    unimplemented!()
}

fn get_pbos(ctx: &Context, target: &Path) -> Result<(), Error> {
    ctx.addons()
        .to_vec()
        .iter()
        .map(|addon| {
            let mut pbo = WritablePbo::new();
            for entry in ctx.fs().join(addon.folder()).unwrap().walk_dir().unwrap() {
                let entry = entry.unwrap();
                if entry.metadata().unwrap().file_type == VfsFileType::File {
                    if entry.filename() == "config.cpp"
                        && entry
                            .parent()
                            .unwrap()
                            .join("config.bin")
                            .unwrap()
                            .exists()
                            .unwrap()
                    {
                        continue;
                    }

                    if FILES.contains(&entry.filename().to_lowercase().as_str()) {
                        let prefix = Prefix::new(
                            &entry.read_to_string().unwrap(),
                            ctx.config().hemtt().pbo_prefix_allow_leading_slash(),
                        )?;
                        pbo.add_extension("prefix", prefix.into_inner());
                    }

                    pbo.add_extension("hemtt", crate::VERSION.to_string());

                    let file = entry
                        .as_str()
                        .trim_start_matches('/')
                        .trim_start_matches(&addon.folder())
                        .trim_start_matches('/')
                        .replace('/', "\\");
                    println!("adding {file} from {}", addon.folder());
                    pbo.add_file(file, entry.open_file().unwrap()).unwrap();
                }
            }
            pbo.write(
                &mut File::create(target.join(format!("{}.pbo", addon.name())))?,
                true,
            )?;
            Ok(())
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(())
}
