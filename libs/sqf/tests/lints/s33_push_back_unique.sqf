private _addons = [];
private _paramAddons = ["a", "b"];

// the ACE3 pattern this came from, both reported
{
    if !(_x in _addons) then {_addons pushBack _x};
    if !(_x in _addons) then {_addons pushBackUnique _x};
} forEach _paramAddons;

// plain variables, reported
if !(_value in _list) then {_list pushBack _value};

// searched array is not the pushed array, ignore
if !(_value in _list) then {_other pushBack _value};

// pushed value is not the searched value, ignore
if !(_value in _list) then {_list pushBack _somethingElse};

// branch does more than push, the guard is not redundant, ignore
if !(_value in _list) then {_list pushBack _value; _count = _count + 1};

// not negated, ignore
if (_value in _list) then {_list pushBack _value};

// has an else branch, ignore
if !(_value in _list) then {_list pushBack _value} else {_list pushBack 0};
