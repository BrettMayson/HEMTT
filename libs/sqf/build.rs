pub fn main() {
    println!("cargo:rerun-if-changed=src/analyze/lints");
}
