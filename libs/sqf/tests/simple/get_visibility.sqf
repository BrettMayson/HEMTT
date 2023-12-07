params ["_arg1", "_arg2"];
if (typeName _arg1 == "OBJECT") then { _arg1 = [eyePos _arg1, _arg1] };
if (typeName _arg2 == "OBJECT") then { _arg2 = [eyePos _arg2, _arg2] };
_arg1 params ["_position1", ["_ignore1", objNull]];
_arg2 params ["_position2", ["_ignore2", objNull]];

private _multiplier = 1 / (2 ^ ((_position1 distance _position2) / 100));
([_ignore1, "VIEW", _ignore2] checkVisibility [_position1, _position2]) * _multiplier
