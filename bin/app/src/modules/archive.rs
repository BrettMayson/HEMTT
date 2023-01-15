use hemtt_bin_error::Error;

use crate::context::Context;

pub fn release(ctx: &Context) -> Result<(), Error> {
    let output = ctx
        .hemtt_folder()
        .parent()
        .expect("HEMTT creates two folders, so a parent should always exist")
        .join(ctx.config().name())
        .with_extension("zip");
    let options = zip::write::FileOptions::default().compression_level(Some(9));

    let mut zip = zip::ZipWriter::new(std::fs::File::create(&output)?);
    for entry in walkdir::WalkDir::new(ctx.hemtt_folder()) {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.is_dir() {
            let path = path
                .strip_prefix(ctx.hemtt_folder())
                .expect("We are in the HEMTT folder, the prefix should always exist")
                .display()
                .to_string();
            if path.is_empty() {
                continue;
            }
            zip.add_directory(path, options)?;
            continue;
        }
        let name = path
            .strip_prefix(ctx.hemtt_folder())
            .expect("We are in the HEMTT folder, the prefix should always exist");
        zip.start_file(name.display().to_string(), options)?;
        std::io::copy(&mut std::fs::File::open(path)?, &mut zip)?;
    }
    zip.finish()?;
    println!("Created release: {output:?}");
    Ok(())
}
