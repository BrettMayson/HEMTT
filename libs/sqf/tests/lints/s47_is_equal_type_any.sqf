private _x = 5;

// chains of isEqualType on the same value, reported
if (_x isEqualType 0 || _x isEqualType "") then { };
if (_x isEqualType 0 || _x isEqualType "" || _x isEqualType objNull) then { };

// short circuited form, still the same pattern
if (_x isEqualType 0 || {_x isEqualType ""}) then { };

// different values compared, ignore
if (_x isEqualType 0 || _y isEqualType "") then { };

// only one comparison, ignore
if (_x isEqualType 0) then { };

// not isEqualType, ignore
if (_x isEqualTo 0 || _x isEqualTo "") then { };
if (_x isEqualType 0 && _x isEqualType "") then { };
