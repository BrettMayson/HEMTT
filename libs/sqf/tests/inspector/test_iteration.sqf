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
