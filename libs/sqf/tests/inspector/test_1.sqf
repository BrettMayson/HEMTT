x setFuel true; // args: takes number 0-1
x setFuel f;
_guy setDamage 1; // _guy undefeind
private _you = player;
_you setDamage 0.5;
_z = 7; // not private
systemChat str _z;
private "_a";
_a = 8;
private ["_b"];
_b = _a + 1;
private _c = _b; // unused
params ["_var1"];
private _var1 = 10; // shadow
diag_log text str _var1;
gx = [];
gx addPublicVariableEventHandler {}; // args: takes lhs string

for "_i" from 1 to 20 step 0.5 do {
    systemChat str _i;
};
for [{private _k = 0}, {_k < 5}, {_k = _k + 1}] do {
    systemChat str _k;
};

//IGNORE_PRIVATE_WARNING["_fromUpper"];
X = _fromUpper;

[] call {
    private "_weird";
    //IGNORE_PRIVATE_WARNING["_weird"] - // No way to know the order is different
    for "_i" from 1 to 5 do {
        if (_i%2 == 0) then {
            truck lock _weird;
        };
        if (_i%2 == 1) then {
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

private _condition =
    {
        [_player, _target] call y
    };
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
    diag_log format ["Key: %1, Value: %2", _key, _value];
};
[_hash, _dumpHash] call CBA_fnc_hashEachPair;

private _myLocalVar1 = 555;
_myLocalVar1 = _myLocalVar1 + 1;
[{
    systemChat str _myLocalVar1; // invalid
}, 0, []] call CBA_fnc_addPerFrameHandler;

private _myLocalVar2 = 55;
[{systemChat str _myLocalVar2}] call CBA_fnc_directCall; // fine
