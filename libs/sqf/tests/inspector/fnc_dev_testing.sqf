params [["_list", [], [[], {}]]];

if (_list isEqualType {}) then {
    _list = [] call {}
} else {
    _list = _list select {!isNull _x};
};
