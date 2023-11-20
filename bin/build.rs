pub fn main() {
    let mut base = env!("CARGO_PKG_VERSION").to_string();
    if option_env!("CI").is_none() {
        base.push_str("-local");
    } else if option_env!("RELEASE").is_none() {
        base.push_str("-dev");
    }
    if cfg!(debug_assertions) {
        base.push_str("-debug");
    }
    println!("cargo:rustc-env=HEMTT_VERSION={base}");
}
