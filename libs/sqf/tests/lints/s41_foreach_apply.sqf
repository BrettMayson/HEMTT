private _values = [];
{
    _values pushBack _x;
} forEach bigArray;

private _values2 = [];
{
    _values2 pushBack _forEachIndex;
} forEach bigArray;

private _values3 = [];
{
    _values3 pushBack (format ["%1", _x]);
} forEach [1, 2, 3];

// ignored because they are assigned to a variable
private _lastValue = { _x } forEach [6,7];
x = { _x + 1 } forEach [1, 2, 3];
