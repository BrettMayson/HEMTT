a setFuel b;
a setFuel 0;
a setFuel true; // invalidArgs: takes number 0-1
_test2 setDamage 1; // undefiend
private _var1 = player;
_var1 setDamage 0.5;

_test3 = 7; // not private
systemChat str _test3;
private "_var2";
_var2 = 8;
private ["_var3"];
_var3 = _var2 + 1;
private _test4 = _var3; // unused
params ["_test5"];
private _test5 = 10; // shadow (same level)
diag_log text str _test5;
gx = [];
gx addPublicVariableEventHandler {}; // args: takes lhs string

for "_var5" from 1 to 20 step 0.5 do {
    systemChat str _var5;
};
for [{private _var6 = 0}, {_var6 < 5}, {_var6 = _var6 + 1}] do {
    systemChat str _var6;
};

//IGNORE_PRIVATE_WARNING["_fromUpper"];
X = _fromUpper;

[] call {
    private "_weird";
    //IGNORE_PRIVATE_WARNING["_weird"] - // No way to know the order is different
    for "_var7" from 1 to 5 do {
        if (_var7%2 == 0) then {
            truck lock _weird;
        };
        if (_var7%2 == 1) then {
            _weird = 0.5;
        };
    };
};

// IGNORE_PRIVATE_WARNING["somePFEH"] - // otherwise will assume it's nil
if (z) then {
    somePFEH = nil;
};
if (y) then {
    setObjectViewDistance somePFEH;
};

somehash getOrDefaultCall ["key", {_test8}, true]; //undefined
private _var8 = objNull;
somehash getOrDefaultCall ["key", {_var8}, true];

// Will have _player and _target
private _condition = { [_player, _target] call y };
[
    "",
    localize "STR_A3_Arsenal",
    "",
    {
        x ctrlSetText _player; // bad arg type
        [_target, _player] call z;
    },
    _condition
] call ace_interact_menu_fnc_createAction;

private _hash = [] call CBA_fnc_hashCreate;
private _dumpHash = {
    // Will have _key and _value
    diag_log format ["Key: %1, Value: %2", _key, _value];
};
[_hash, _dumpHash] call CBA_fnc_hashEachPair;

private _test9 = 555;
_test9= _test9 + 1;
[{
    systemChat str _test9; // invalid
}, 0, []] call CBA_fnc_addPerFrameHandler;

private _var9 = 55;
[{systemChat str _var9}] call CBA_fnc_directCall;

 // Will have _x
filter = [orig, {_x + 1}] call CBA_fnc_filter;

private _var10 = 123;
[player, {x = _var10}] call ace_common_fnc_cachedcall;

for "_test10" from 1 to 1 step 0.1 do {};
[5] params ["_test11"];
