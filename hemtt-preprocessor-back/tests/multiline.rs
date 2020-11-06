#[test]
fn basic() {
    let content = 
r#"#define QUOTE(s) #s
#define my_class class my_thing {\
    name = QUOTE(hello world); \
}

my_class;
"#;
    let processed = hemtt_preprocessor::Preprocessor::from_source(&content);
    assert_eq!(r#"
class my_thing {
    name = "hello world";
};
"#, processed.output);
}

#[test]
fn define_and_call_nested() {
    let content = 
r#"#define ADDON test
#define GVAR(var1) DOUBLES(ADDON,var1)
#define DOUBLES(var1,var2) var1##_##var2
#define QUOTE(s) #s
#define my_class class my_thing {\
    name = QUOTE(GVAR(bar)); \
}

my_class;
"#;
    let processed = hemtt_preprocessor::Preprocessor::from_source(&content);
    assert_eq!(r#"
class my_thing {
    name = "test_bar";
};
"#, processed.output);
}

#[test]
fn define_and_call_nested_comma() {
    let content = 
r#"#define ADDON test
#define DOUBLES(var1,var2) var1##_##var2
#define QUOTE(s) #s
#define my_class class my_thing {\
    name = QUOTE(DOUBLES(test,bar)); \
}

my_class;
"#;
    let processed = hemtt_preprocessor::Preprocessor::from_source(&content);
    assert_eq!(r#"
class my_thing {
    name = "test_bar";
};
"#, processed.output);
}

#[test]
fn define_and_call_nested_sqf_call() {
    let content = 
r#"#define ADDON test
#define DOUBLES(var1,var2) var1##_##var2
#define QUOTE(s) #s
#define my_class class my_thing {\
    name = QUOTE([DOUBLES(test,bar), data] call _my_fnc); \
}

my_class;
"#;
    let processed = hemtt_preprocessor::Preprocessor::from_source(&content);
    assert_eq!(r#"
class my_thing {
    name = "[test_bar, data] call _my_fnc";
};
"#, processed.output);
}

#[test]
fn define_and_call_behind_word() {
    let content = 
r#"#define ADDON test
#define DOUBLES(var1,var2) var1##_##var2
#define QUOTE(s) #s
#define VALUES DOUBLES(ADDON,data)
#define CALL(fnc) [VALUES] call fnc
#define my_class class my_thing {\
    name = QUOTE(CALL(something)); \
}

my_class;
"#;
    let processed = hemtt_preprocessor::Preprocessor::from_source(&content);
    assert_eq!(r#"
class my_thing {
    name = "[test_data] call something";
};
"#, processed.output);
}

#[test]
fn define_and_call_behind_word_multiline() {
    let content = 
r#"#define ADDON test
#define DOUBLES(var1,var2) var1##_##var2
#define QUOTE(s) #s
#define VALUES \
    DOUBLES(\
        ADDON,\
        data\
    )
#define CALL(fnc) [VALUES] call fnc
#define my_class class my_thing {\
    name = QUOTE(CALL(something)); \
}

my_class;
"#;
    let processed = hemtt_preprocessor::Preprocessor::from_source(&content);
    assert_eq!(r#"
class my_thing {
    name = "[test_data] call something";
};
"#, processed.output);
}

#[test]
fn multiline() {
    let content = 
r#"#define QUOTE(var1) #var1 
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
};
"#;
    let processed = hemtt_preprocessor::Preprocessor::from_source(&content);
    assert_eq!(r#"

class CfgPatches {
    class q {
        expression = "if (_value != (if (isNumber (configFile >> 'CfgVehicles' >> typeOf _this >> ""test_fuelCargo"")) then {getNumber (configFile >> 'CfgVehicles' >> typeOf _this >> ""test_fuelCargo"")} else {(if (0 < getNumber (configFile >> 'CfgVehicles' >> typeOf _this >> 'transportFuel')) then {getNumber (configFile >> 'CfgVehicles' >> typeOf _this >> 'transportFuel')} else {-1})})) then {[_this,_value] call test_fnc_makeSource}";
    };
};
"#, processed.output);
}
