#[test]
fn cba_special() {
    for char in [
        "²", "ƒ", "‡", "Œ", "Š", "–", "µ", "œ", "š", "ˆ", "˜", "€", "º", "¨", "¬",
    ]
    .iter()
    {
        hemtt_preprocessor::parse("special", char, &None).unwrap();
    }
}
