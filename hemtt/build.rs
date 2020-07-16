fn main() {
    peg::cargo_build("src/grammars/config.rustpeg");
    peg::cargo_build("src/grammars/preprocess.rustpeg");
}
