use std::fs::read_to_string;

fn resolver(name: &str) -> String {
    read_to_string(format!("tests/preprocess/{}", name)).unwrap()
}

#[test]
fn define() {
    let content = r#"
#define affirmative true
value = affirmative;
"#;
    let config =
        hemtt_arma_config::preprocess(hemtt_arma_config::tokenize(content).unwrap(), &resolver);
    let config = hemtt_arma_config::render(config.unwrap());
    println!("======");
    println!("{}", config);
    println!("======");
    assert_eq!("\nvalue = true;\n", config);
}

#[test]
fn nested_define() {
    let content = r#"
#define TEST 123
#define SOMETHING TEST

value = SOMETHING;
"#;
    let config =
        hemtt_arma_config::preprocess(hemtt_arma_config::tokenize(content).unwrap(), &resolver);
    let config = hemtt_arma_config::render(config.unwrap());
    println!("======");
    println!("{}", config);
    println!("======");
    assert_eq!("\n\nvalue = 123;\n", config);
}

#[test]
fn undefine() {
    let content = r#"
#define affirmative true
value = affirmative;
#undef affirmative
#ifdef affirmative
defined = true;
#else
defined = false;
#endif
"#;
    let config =
        hemtt_arma_config::preprocess(hemtt_arma_config::tokenize(content).unwrap(), &resolver);
    let config = hemtt_arma_config::render(config.unwrap());
    println!("======");
    println!("{}", config);
    println!("======");
    assert_eq!("\nvalue = true;\n\n\ndefined = false;\n\n", config);
}

#[test]
fn define_call() {
    let content = r#"
#define SAY_HI(NAME) Hi NAME

value = "SAY_HI(Brett)";
"#;
    let config =
        hemtt_arma_config::preprocess(hemtt_arma_config::tokenize(content).unwrap(), &resolver);
    let config = hemtt_arma_config::render(config.unwrap());
    println!("======");
    println!("{}", config);
    println!("======");
    assert_eq!("\n\nvalue = \"Hi Brett\";\n", config);
}

#[test]
fn recursive() {
    let content = r#"
#define ADD_PERIOD(NAME) NAME.
#define MR(NAME) Mr. ADD_PERIOD(NAME)
#define SAY_HI(NAME) Hi MR(NAME)

value = "SAY_HI(Brett)";
"#;
    let config =
        hemtt_arma_config::preprocess(hemtt_arma_config::tokenize(content).unwrap(), &resolver);
    let config = hemtt_arma_config::render(config.unwrap());
    println!("======");
    println!("{}", config);
    println!("======");
    assert_eq!("\n\nvalue = \"Hi Mr. Brett.\";\n", config);
}

#[test]
fn recursive2() {
    let content = r#"
#define ADD_PERIOD(NAME) NAME.
#define MR(NAME) Mr. NAME
#define SAY_HI(NAME) Hi MR(ADD_PERIOD(NAME))

value = "SAY_HI(Brett)";
"#;
    let config =
        hemtt_arma_config::preprocess(hemtt_arma_config::tokenize(content).unwrap(), &resolver);
    let config = hemtt_arma_config::render(config.unwrap());
    println!("======");
    println!("{}", config);
    println!("======");
    assert_eq!("\n\nvalue = \"Hi Mr. Brett.\";\n", config);
}

#[test]
fn recursive_quote() {
    let content = r#"
#define QUOTE(var1) #var1
#define DOUBLES(var1,var2) var1##_##var2
#define ADDON test
#define GVAR(var1) DOUBLES(ADDON,var1)
#define QGVAR(var1) QUOTE(GVAR(var1))
#define QQGVAR(var1) QUOTE(QGVAR(var1))

value = GVAR(myVar);
value = QGVAR(myVar);
value = QUOTE(My variable is QQGVAR(myVar));
"#;
    let config =
        hemtt_arma_config::preprocess(hemtt_arma_config::tokenize(content).unwrap(), &resolver);
    let config = hemtt_arma_config::render(config.unwrap());
    println!("======");
    println!("{}", config);
    println!("======");
    assert_eq!("\n\nvalue = test_myVar;\nvalue = \"test_myVar\";\nvalue = \"My variable is \"\"test_myVar\"\"\";\n", config);
}

#[test]
fn nested() {
    let content = r#"#define QUOTE(var1) #var1
#define B(x) {x}
#define C ({1})

class CfgPatches {
    class q {
        t = QUOTE(B(C); call f);
    };
};"#;
    let config =
        hemtt_arma_config::preprocess(hemtt_arma_config::tokenize(content).unwrap(), &resolver);
    let config = hemtt_arma_config::render(config.unwrap());
    println!("======");
    println!("{}", config);
    println!("======");
    assert_eq!(
        "\nclass CfgPatches {\n    class q {\n        t = \"{({1})}; call f\";\n    };\n};",
        config
    );
}

#[test]
fn very_nested() {
    let content = r#"#define QUOTE(var1) #var1
#define ARR_2(ARG1,ARG2) ARG1, ARG2
#define DOUBLES(var1,var2) var1##_##var2
#define TRIPLES(var1,var2,var3) var1##_##var2##_##var3
#define ADDON test
#define DFUNC(var1) TRIPLES(ADDON,fnc,var1)
#define GVAR(var1) DOUBLES(ADDON,var1)
#define QGVAR(var1) QUOTE(GVAR(var1))
#define QQGVAR(var1) QUOTE(QGVAR(var1))

#define GET_NUMBER(config,default) (if (isNumber (config)) then {getNumber (config)} else {default})
#define GET_NUMBER_GREATER_ZERO(config,default) (if (0 < getNumber (config)) then {getNumber (config)} else {default})
#define DEFAULT_FUELCARGO \
    GET_NUMBER(\
        configFile >> 'CfgVehicles' >> typeOf _this >> QQGVAR(fuelCargo),\
        GET_NUMBER_GREATER_ZERO(configFile >> 'CfgVehicles' >> typeOf _this >> 'transportFuel',-1)\
    )

class CfgPatches {
    class q {
        expression = QUOTE(if (_value != DEFAULT_FUELCARGO) then {[ARR_2(_this,_value)] call DFUNC(makeSource)});
    };
};"#;
    let config =
        hemtt_arma_config::preprocess(hemtt_arma_config::tokenize(content).unwrap(), &resolver);
    let config = hemtt_arma_config::render(config.unwrap());
    println!("======");
    println!("{}", config);
    println!("======");
    assert_eq!(
        r#"

class CfgPatches {
    class q {
        expression = "if (_value != (if (isNumber (configFile >> 'CfgVehicles' >> typeOf _this >> ""test_fuelCargo"")) then {getNumber (configFile >> 'CfgVehicles' >> typeOf _this >> ""test_fuelCargo"")} else {(if (0 < getNumber (configFile >> 'CfgVehicles' >> typeOf _this >> 'transportFuel')) then {getNumber (configFile >> 'CfgVehicles' >> typeOf _this >> 'transportFuel')} else {-1})})) then {[_this, _value] call test_fnc_makeSource}";
    };
};"#,
        config
    );
}

#[test]
fn include() {
    let config = hemtt_arma_config::preprocess(
        hemtt_arma_config::tokenize(&read_to_string("tests/preprocess/base.hpp").unwrap()).unwrap(),
        &resolver,
    );
    let config = hemtt_arma_config::render(config.unwrap());
    println!("======");
    println!("{}", config);
    println!("======");
    assert_eq!(
        r#"
class included {};

class test {
    value = 100;
};
"#,
        config
    );
}

#[test]
fn unknown() {
    let content = r#"
#notreal affirmative true
value = affirmative;
"#;
    let config =
        hemtt_arma_config::preprocess(hemtt_arma_config::tokenize(content).unwrap(), &resolver);
    let config = hemtt_arma_config::render(config.unwrap());
    println!("======");
    println!("{}", config);
    println!("======");
    assert_eq!("\nvalue = affirmative;\n", config);
}
