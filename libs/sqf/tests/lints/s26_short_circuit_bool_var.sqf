private _test1 = false;
private _test2 = true;

if (_test1 && alive player) then { };
if (_test1 && _test2) then { };
if (_test1 && {call x; _test2}) then { };
if (_test1 && {_test2}) then { };

if (!isNil "someVar" && {somevar}) then {}; // ignore

// comparisons of simple values, all reported
if (_test1 && {_a isEqualTo _b}) then { };
if (_test1 && {_a isNotEqualTo "text"}) then { };
if (_test1 && {_a == _b}) then { };
if (_test1 && {_a != 5}) then { };
if (_test1 && {_a > 5}) then { };
if (_test1 && {_a < _b}) then { };
if (_test1 && {_a >= 5}) then { };
if (_test1 && {_a <= _b}) then { };
if (_test1 || {_a isEqualTo _b}) then { };
if (_test1 && {_a isEqualRef _b}) then { };
if (_test1 && {_a isNotEqualRef _b}) then { };
if (_test1 && {_a isEqualType _b}) then { };

// the short circuit is guarding a command, ignore
if (count _array > 0 && {_array select 0 isEqualTo "x"}) then { };
if (!isNull _obj && {getPosATL _obj isEqualTo []}) then { };
if (_test1 && {alive player isEqualTo true}) then { };

// not a comparison, ignore
if (_test1 && {_a + _b}) then { };
if (_test1 && {_a isKindOf "Man"}) then { };
