use hemtt_sign::*;

fn main() {
    crate::execute(&std::env::args().collect::<Vec<_>>()).unwrap();
}
