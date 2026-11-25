use std::path::PathBuf;

use crate::Error;

#[derive(clap::Args)]
pub struct JsonArgs {
    /// P3d to convert
    p3d: String,
    /// Where to save the file
    output: String,
}

/// Execute the json command
///
/// # Errors
/// [`Error`] depending on the modules
pub fn execute(args: &JsonArgs) -> Result<(), Error> {
    let p3d = PathBuf::from(&args.p3d);
    let output = PathBuf::from(&args.output);
    if output.exists() {
        error!("Output file already exists");
        return Ok(());
    }
    let p3d = hemtt_p3d::P3D::read(&mut fs_err::File::open(p3d)?)?;
    let _ = fs_err::create_dir_all(
        output
            .parent()
            .expect("Output file has no parent directory"),
    );
    serde_json::to_writer(fs_err::File::create(output)?, &p3d)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{JsonArgs, execute};
    use crate::Error;

    #[test]
    fn binarized_p3d_is_an_error() {
        let _directory = hemtt_test::directory::TemporaryDirectory::new();
        let mut odol = b"ODOL".to_vec();
        odol.extend_from_slice(&[0; 64]);
        fs_err::write("model.p3d", odol).expect("write p3d");

        let result = execute(&JsonArgs {
            p3d: "model.p3d".to_string(),
            output: "out/model.json".to_string(),
        });
        assert!(
            matches!(result, Err(Error::P3d(hemtt_p3d::Error::BinarizedP3D))),
            "expected a binarized error, got {result:?}"
        );
        assert!(!std::path::Path::new("out/model.json").exists());
    }
}
