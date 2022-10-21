#define QUOTE(var1) #var1
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
