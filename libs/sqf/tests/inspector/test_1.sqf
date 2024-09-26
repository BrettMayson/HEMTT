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
