use std::process::Command;
fn main() {
    let appveyor = option_env!("APPVEYOR_REPO_TAG");
    let travis = option_env!("TRAVIS_TAG");
    if (travis.is_none()) && (appveyor.is_none() || appveyor.unwrap() == "false") {
        let output = Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output();
        if let Ok(output) = output {
            let git_hash = String::from_utf8(output.stdout).unwrap();
            println!("cargo:rustc-env=GIT_HASH={}", git_hash);
        }
    }
}
