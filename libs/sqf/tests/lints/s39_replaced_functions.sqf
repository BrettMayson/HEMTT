[1,2,3] call BIS_fnc_selectRandom;

[[1,2,3], 3] call BIS_fnc_vectorMultiply;

private _parent = [_this, 0, [], [[]]] call BIS_fnc_param;
private _child = [_parent, 1, "", [""]] call BIS_fnc_paramIn;
