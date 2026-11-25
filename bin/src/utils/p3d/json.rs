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
        return Err(Error::OutputExists(args.output.clone()));
    }
    let p3d = hemtt_p3d::P3D::read(&mut fs_err::File::open(p3d)?)?;
    if let Some(parent) = output.parent()
        && !parent.as_os_str().is_empty()
    {
        fs_err::create_dir_all(parent)?;
    }
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

    fn valid_p3d() -> Vec<u8> {
        fs_err::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../libs/p3d/tests/ace_gunbag.p3d"
        ))
        .expect("read test p3d")
    }

    #[test]
    fn existing_output_is_an_error() {
        let _directory = hemtt_test::directory::TemporaryDirectory::new();
        fs_err::write("model.p3d", valid_p3d()).expect("write p3d");
        fs_err::write("model.json", "keep").expect("write output");

        let result = execute(&JsonArgs {
            p3d: "model.p3d".to_string(),
            output: "model.json".to_string(),
        });
        assert!(
            matches!(&result, Err(Error::OutputExists(path)) if path == "model.json"),
            "expected an output exists error, got {result:?}"
        );
        assert_eq!(fs_err::read_to_string("model.json").expect("read"), "keep");
    }

    #[test]
    fn output_without_parent_folder() {
        let _directory = hemtt_test::directory::TemporaryDirectory::new();
        fs_err::write("model.p3d", valid_p3d()).expect("write p3d");

        execute(&JsonArgs {
            p3d: "model.p3d".to_string(),
            output: "model.json".to_string(),
        })
        .expect("convert");
        let json: serde_json::Value =
            serde_json::from_str(&fs_err::read_to_string("model.json").expect("read"))
                .expect("valid json");
        assert!(json.get("lods").is_some());
    }
}
