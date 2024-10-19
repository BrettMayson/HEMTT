private _x = 10;
private _y = true;
private _z = false;

if (_context != 2 && {_context == 4 || _newDamage == 0}) exitWith {};

if ((!(_x isEqualType 0)) || {_x < 0 || _x > 1}) exitWith {};

if ({_y && _x < 10} || {_z && _x > 30 && {_x < 10}}) then {};

if (_x < 20 && {_x > 30 && {_y || _z}}) then {
    systemChat "This will never be executed";
};

if (_x < 20 && _x < 10) then {
    systemChat "This is wasteful";
};

if (_key != 1 && _key != 2 && _key != 3) exitWith {false};
