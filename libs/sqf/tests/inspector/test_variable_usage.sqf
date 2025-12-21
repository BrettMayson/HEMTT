private _testA = "5";
_testA = parseNumber _testA;
systemChat _testA; // error (var is known to be number here)

private _testB = "5";
if (x) then { _testB = parseNumber _testB; };
systemChat _testB; // ok (var may still be string here)


// overwritten before use
private _testC = "5";
_testC = "7"; // unused (overwrite)
systemChat _testC;

private ["_testD"];
_testD = "5"; // safe
systemChat _testD;

private "_testE";
_testE = "5"; // safe
systemChat _testE;

{
    private _x = 6; // safe because _x is magic from forEach
} forEach [1];

[a,b,c,d,{ // magic (_target, _player) from ace_interact_menu_fnc_createaction ignored
    params ["_target"];
    _target setDamage 1;
},e,f,g,h] call ace_interact_menu_fnc_createaction;


// single-ended-if assignment
private _testF = if (x) then { 5 };
systemChat str _testF;  // assume if result is nil

uiNamespace setVariable ["why", drawIcon3D someData]; // no reason to assign nil from a command
uiNamespace setVariable ["ok", nil]; // explicit nil is fine
