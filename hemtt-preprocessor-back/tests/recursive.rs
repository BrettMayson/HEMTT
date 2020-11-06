#[test]
fn recursive() {
    let content = 
r#"#define QUOTE(var1) #var1 
#define B(x) {x}
#define C ({1})

class CfgPatches {
    class q {
        t = QUOTE(B(C); call f);
    };
};
"#;
    let processed = hemtt_preprocessor::Preprocessor::from_source(&content);
    assert_eq!(r#"
class CfgPatches {
    class q {
        t = "{({1})}; call f";
    };
};
"#, processed.output);
}
