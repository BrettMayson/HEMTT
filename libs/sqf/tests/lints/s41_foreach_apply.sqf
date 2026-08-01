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
