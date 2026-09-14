use std::path::PathBuf;

pub fn main() {
    println!("cargo:rerun-if-changed=src/analyze/lints");
    println!("cargo:rerun-if-changed=c19_classes.txt");

    let input = fs_err::read_to_string("src/analyze/lints/c19_classes.txt")
        .expect("Failed to read c19_classes.txt");

    let classes = input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with("//"))
        .map(|line| format!("{line:?},"))
        .collect::<Vec<_>>();

    let output = format!("{}{}{}", "[\n", classes.join("\n"), "\n]");

    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("Failed to get OUT_DIR"));
    fs_err::write(out_dir.join("c19_classes.rs"), output).expect("Failed to write c19_classes.rs");
}
