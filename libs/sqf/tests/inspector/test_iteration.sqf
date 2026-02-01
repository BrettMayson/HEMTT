private _a1 = [1,2,3];
{ systemChat format ["Value: %1", _x]; } forEach _a1;
{ systemChat format ["Value: %1, Key %2", _x, _y]; } forEach _a1; // _y is invalid (set is an array)
_a1 apply { systemChat format ["Value: %1, Key %2", _x, _y]; }; // _y is invalid (set is an array)
_a1 findIf { systemChat str _forEachIndex; _x > 2 }; // _forEachIndex is invalid
{ _x + "A" } forEach _a1; // invalid cmd:+ (_x is a number)
_a1 findIf { _x + "B" };  // invalid cmd:+ (_x is a number)
{ _x + "C" } forEach ["B", 1]; // "ok"

private _a2 = xUnknown;
{ systemChat format ["Value: %1, Key %2", _x, _y]; } forEach _a2;
_a2 apply { systemChat format ["Value: %1, Key %2", _x, _y]; };
_a2 findIf { systemChat str _forEachIndex; false }; // _forEachIndex is invalid
_a2 findIf { _y isNotEqualTo 5 }; // _y is invalid

private _a3 = createHashMap;
_a3 set ["a", 1];
{ systemChat format ["Value: %1, Key %2", _x, _y]; } forEach _a3;
{ systemChat format ["Value: %1, Key %2", _x, _y]; } forEachReversed _a3; // invalid cmd:forEachReversed
_a3 apply { systemChat format ["Value: %1, Key %2", _x, _y]; };

private _a4 = [1];
_a4 resize 0;
_a4 pushBack "x";
{ _x + "D" } forEach _a4; // ok


// "test runs" for loop iterations
private _oldLoc = false;
{
    _x params ["_locNew", "_colorNew"];
    if (_oldLoc isNotEqualTo false) then {
        drawLine3D [_oldLoc, _locNew, _colorNew]; // safe because it happens inside an inner if-scope
    };
    _oldLoc = _locNew;
} forEach x_points;

private _oldLoc2 = false;
{
    _x params ["_locNew", "_colorNew"];
    drawLine3D [_oldLoc2, _locNew, _colorNew]; // error
    _oldLoc2 = _locNew;
} forEach x_points;

private _z = "5"; 
for "_i" from 1 to 11 do {
    if (_i % 2 == 0) then { 
        _z = str (_z / 10); // safe
    };
    if (_i % 2 == 1) then { 
        _z = parseNumber (_z + "0"); 
    }; 
};

private "_someVar";
[1,2,3] select {
    if (!isNil "_someVar") then {
        x = (_x + _someVar) > 5 // safe
    };
    _someVar = _x;
    false;
};