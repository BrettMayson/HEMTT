use std::fs::create_dir_all;

use hemtt_bin_error::Error;

use crate::context::Context;

pub fn release(ctx: &Context) -> Result<(), Error> {
    let output = ctx.project_folder().join("releases");
    if !output.exists() {
        create_dir_all(&output)?;
    }
    let output = output
        .join(format!("{}-latest", ctx.config().name()))
        .with_extension("zip");
    let options = zip::write::FileOptions::default().compression_level(Some(9));

    let mut zip = zip::ZipWriter::new(std::fs::File::create(&output)?);
    for entry in walkdir::WalkDir::new(ctx.out_folder()) {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.is_dir() {
            let path = path
                .strip_prefix(ctx.out_folder())
                .expect("We are in the HEMTT folder, the prefix should always exist")
                .display()
                .to_string();
            if path.is_empty() {
                continue;
            }
            zip.add_directory(
                format!("@{}/{}", ctx.config().prefix(), path.replace('\\', "/")),
                options,
            )?;
            continue;
        }
        let name = path
            .strip_prefix(ctx.out_folder())
            .expect("We are in the HEMTT folder, the prefix should always exist");
        zip.start_file(
            format!(
                "@{}/{}",
                ctx.config().prefix(),
                name.display().to_string().replace('\\', "/")
            ),
            options,
        )?;
        std::io::copy(&mut std::fs::File::open(path)?, &mut zip)?;
    }
    zip.finish()?;
    println!("Created release: {}", output.display());
    std::fs::copy(&output, {
        let mut output = output.clone();
        output.set_file_name(format!(
            "{}-{}.zip",
            ctx.config().name(),
            ctx.config().version().get()?
        ));
        output
    })?;
    Ok(())
}
