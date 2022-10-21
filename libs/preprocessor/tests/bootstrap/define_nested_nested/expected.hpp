

class CfgPatches {
    class q {
        expression = "if (_value != 
    (if (isNumber (
        configFile >> 'CfgVehicles' >> typeOf _this >> ""test_fuelCargo"")) then {getNumber (
        configFile >> 'CfgVehicles' >> typeOf _this >> ""test_fuelCargo"")} else {
        (if (0 < getNumber (configFile >> 'CfgVehicles' >> typeOf _this >> 'transportFuel')) then {getNumber (configFile >> 'CfgVehicles' >> typeOf _this >> 'transportFuel')} else {-1})
})) then {[_this, _value] call test_fnc_makeSource}";
    };
};
