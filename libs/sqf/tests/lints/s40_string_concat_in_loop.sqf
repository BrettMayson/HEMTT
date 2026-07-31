private _myString = "";
for "_i" from 0 to 10000 do {
    _myString = _myString + "123";
};

private _myString2 = "";
for [{ _i = 0 }, { _i < 10000 }, { _i = _i + 1 }] do {
    _myString2 = _myString2 + "123";
};

private _smallString = "";
for "_i" from 0 to 10 do {
    _smallString = _smallString + "x";
};

private _names = "";
GlobalNames apply {
    _names = _names + _x;
};

private _addonsList = [];
(entities "all") apply {
    _addonsList = _addonsList + (unitAddons typeOf _x);
};

private _addonsList = "";
(entities "all") apply {
    _addonsList = _addonsList + format["%1, ", unitAddons typeOf _x];
};
