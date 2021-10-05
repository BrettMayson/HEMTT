use hemtt_sign::*;

fn main() {
    crate::execute("bisign", &std::env::args().collect::<Vec<_>>()).unwrap();
}
