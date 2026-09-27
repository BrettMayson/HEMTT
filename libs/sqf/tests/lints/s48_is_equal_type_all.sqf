private _arr = [1, 2, 3];

// count pattern, reported
if ({_x isEqualType 0} count _arr == count _arr) then { };
if (count _arr == {_x isEqualType 0} count _arr) then { };
if ({_x isEqualType ""} count _arr isEqualTo count _arr) then { };

// findIf pattern, reported
if (_arr findIf {!(_x isEqualType 0)} == -1) then { };
if (_arr findIf {!(_x isEqualType 0)} < 0) then { };
if (_arr findIf {!(_x isEqualType 0)} <= -1) then { };
if (-1 == _arr findIf {!(_x isEqualType 0)}) then { };
if (0 > _arr findIf {!(_x isEqualType 0)}) then { };

// negated forms, reported as !(... isEqualTypeAll ...)
if ({_x isEqualType 0} count _arr != count _arr) then { };
if (_arr findIf {!(_x isEqualType 0)} != -1) then { };
if (_arr findIf {!(_x isEqualType 0)} > -1) then { };
if (_arr findIf {!(_x isEqualType 0)} >= 0) then { };
if (0 <= _arr findIf {!(_x isEqualType 0)}) then { };

// counting a different array, ignore
if ({_x isEqualType 0} count _arr == count _other) then { };

// not the magic _x, ignore
if ({_y isEqualType 0} count _arr == count _arr) then { };

// findIf without the negation is a different question, ignore
if (_arr findIf {_x isEqualType 0} == -1) then { };

// sentinel that does not correspond to a -1 test, ignore
if (_arr findIf {!(_x isEqualType 0)} == 2) then { };
if (_arr findIf {!(_x isEqualType 0)} > 0) then { };

// unrelated comparisons, ignore
if (count _arr == 0) then { };
if ({_x > 2} count _arr == count _arr) then { };
