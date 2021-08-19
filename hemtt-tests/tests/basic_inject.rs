#[test]
fn basic() {
    let source = r#"systemChat "Hello, world!";
if (player isEqualTo vehicle player) then {
    systemChat "You are not in a vehicle";
};
"#;
    let injected = hemtt_tests::inject(source, "key");
    assert_eq!(injected.0, r#""hemtt_tests" callExtension ["cov", ["key", 0]];systemChat "Hello, world!";
"hemtt_tests" callExtension ["cov", ["key", 1]];
if (player isEqualTo vehicle player) then {
"hemtt_tests" callExtension ["cov", ["key", 2]];
    systemChat "You are not in a vehicle";
"hemtt_tests" callExtension ["cov", ["key", 3]];
};
"hemtt_tests" callExtension ["cov", ["key", 4]];
"#);
}

#[test]
fn arma_loop() {
    let source = 
r#"{
    systemChat format ["Unit: %1", _x];
} forEach allUnits;
"#;
    let injected = hemtt_tests::inject(source, "key");
    assert_eq!(injected.0, r#""hemtt_tests" callExtension ["cov", ["key", 0]];{
"hemtt_tests" callExtension ["cov", ["key", 1]];
    systemChat format ["Unit: %1", _x];
"hemtt_tests" callExtension ["cov", ["key", 2]];
} forEach allUnits;
"hemtt_tests" callExtension ["cov", ["key", 3]];
"#);
}
