use vfs::MemoryFS;

#[test]
fn cba_special() {
    for char in [
        "²", "ƒ", "‡", "Œ", "Š", "–", "µ", "œ", "š", "ˆ", "˜", "€", "º", "¨", "¬",
    ]
    .iter()
    {
        hemtt_preprocessor::parse(&MemoryFS::new().into(), char, &None).unwrap();
    }
}
