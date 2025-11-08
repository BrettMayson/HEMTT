// _var[LETTER] are safe
// _test[NUMBER] are errors

a setFuel b;
a setFuel 0;
a setFuel true; // invalidArgs: takes number 0-1
_test2 setDamage 1; // undefiend
private _varA = player;
_varA setDamage 0.5;

_test3 = 7; // not private
systemChat str _test3;
private "_varB";
_varB = 8;
private ["_varC"];
_varC = _varB + 1;
private _test4 = _varC; // unused
params ["_test5"];
private _test5 = 10; // shadow (same level)
diag_log text str _test5;
gx = [];
gx addPublicVariableEventHandler {}; // args: takes lhs string

for "_varE" from 1 to 20 step 0.5 do {
    systemChat str _varE;
};
for [{private _varF = 0}, {_varF < 5}, {_varF = _varF + 1}] do {
    systemChat str _varF;
};

#pragma hemtt ignore_variables ["_fromUpper"]
X = _fromUpper;

[] call {
    private "_weird";
    #pragma hemtt ignore_variables ["_weird"]
    for "_varG" from 1 to 5 do {
        if (_varG%2 == 0) then {
            truck lock _weird;
        };
        if (_varG%2 == 1) then {
            _weird = 0.5;
        };
    };
};

#pragma hemtt ignore_variables ["somePFEH"]
if (z) then {
    somePFEH = nil; // otherwise will assume it's always nil
};
if (y) then {
    setObjectViewDistance somePFEH;
};

somehash getOrDefaultCall ["key", {_test8}, true]; //undefined
private _varH = objNull;
somehash getOrDefaultCall ["key", {_varH}, true];

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
    _condition,
    {systemChat str _player; []}
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

private _varI = 55;
[{systemChat str _varI}] call CBA_fnc_directCall;

 // Will have _x
filter = [orig, {_x + 1}] call CBA_fnc_filter;

private _varJ = 123;
[player, {x = _varJ}] call ace_common_fnc_cachedcall;

for "_test10" from 1 to 1 step 0.1 do {};
[5] params ["_test11"];

params [["_varK", objNull, [objNull]]];
{
    private _varName = vehicleVarName _varK;
    _varK setVehicleVarName (_varName + "ok");
} call CBA_fnc_directCall;

_this select 0 drawIcon [ // invalidArgs
    "#(rgb,1,1,1)color(1,1,1,1)",
    [0,1,0,1],
    player,
    0,
    0,
    0,
    5555 // text - optional <string>
];

private _varL = nil;
call ([{_varL = 0;}, {_varL = 1;}] select (x == 1)); 
["A", "B"] select _varL;

params xParams;
params [];
params [[""]];
params [["_varM", "", [""]], ["_varN", 1, [0]], ["_varO", { systemChat _varM }]];
_varM = _varM + "";
_varN = _varN + 2;
call _varO;

params [["_someString", "abc", [""]], ["_someCode", { 60 setGusts _someString }]];
call _someCode; // InvalidArgs for setGusts

// ensure we use a generic version of the array param types or format would have an error
params [["_varP", "", ["", []]]];
format _varP;


[{
    [_test12] call some_func; // undef, not orphan because CBA_fnc_execNextFrame is a known clean scope 
}, player] call CBA_fnc_execNextFrame;
[{
    [_test13] call some_func; // undef, is orphan
}, player] call unknown_fnc_Usage;

private _test14 = str 12345678 splitString "5";
if (count _test14 == 0) then { call b };

player addEventHandler ["InventoryClosed", {
    missionNamespace setVariable ["test", nil ];
    player removeEventHandler [_thisEvent, _thisEventhandler]; // magic vars
}];

addMissionEventHandler ["EachFrame", { systemChat str [_thisArgs, time] }, [time]];

#pragma hemtt ignore_variables ["_varR"]
{
    private _varR = _x;
} forEach [1,2,3];

if !(isNil "_fnc_scriptNameParent") then {
    diag_log format["[x] Function called with a nil value from script: %1",_fnc_scriptNameParent];
};
if !(isNil "_fnc_scriptName") then {
    diag_log format["[x] Function called with a nil value from script: %1",_fnc_scriptName];
};

switch (getNumber (someConfig >> "ItemInfo" >> "type")) do {
    case 1: { false; };
    case 2: { false; };
    case default { true }; // error, using nil from default
};

private _test15 = [] call { 5 };
{} forEach _test15; // error, _test15 is number

private _varS = call {
    if (true) exitWith { 6 };
};
_varS + 1;

private _test16 = [] call { xxx = 1};
hashValue _test16; // error, _test16 is assignment
hashValue nil; // explicit nil is allowed

private _varT = switch (test_var) do {
    case 1: { {objNull} };
    default { {displayNull} };
};
sin (call _varT); // error rhs is object/display
