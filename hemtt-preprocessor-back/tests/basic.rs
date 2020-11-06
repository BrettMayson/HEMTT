#[test]
fn define() {
    let content = r#"
#define affirmative true
value = affirmative;
"#;
    assert_eq!(
r#"
value = true;
"#, hemtt_preprocessor::Preprocessor::from_source(content).output);
}

#[test]
fn quote() {
    let content = r#"
#define Q(s) #s
value = Q(hello world);
"#;
    assert_eq!(
r#"
value = "hello world";
"#, hemtt_preprocessor::Preprocessor::from_source(content).output);
}
