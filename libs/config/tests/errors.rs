use chumsky::Parser;

#[test]
fn unquoted() {
    let input = r#"class CfgPatches {
    class Test {
        value = hello world;
    };
};"#;
    let parsed = hemtt_config::parse::config().parse_recovery(input);
    println!("{:#?}", parsed.0);
}
