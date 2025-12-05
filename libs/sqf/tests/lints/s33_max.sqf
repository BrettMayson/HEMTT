// Should trigger warning: if (x > y) then {x} else {y}
private _x = 5;
private _y = 10;
private _max1 = if (_x > _y) then {_x} else {_y};

// Should trigger warning: if (x >= y) then {x} else {y}
private _max2 = if (_x >= _y) then {_x} else {_y};

// Should trigger warning: if (x < y) then {y} else {x} (reversed)
private _max3 = if (_x < _y) then {_y} else {_x};

// Should trigger warning: if (x <= y) then {y} else {x} (reversed)
private _max4 = if (_x <= _y) then {_y} else {_x};

// Should trigger warning: [_x, _y] select (_x < _y)
private _max5 = [_x, _y] select (_x < _y);

// Should NOT trigger: different variables
private _z = 3;
private _notMax1 = if (_x > _y) then {_x} else {_z};

// Should NOT trigger: branches don't match pattern
private _notMax2 = if (_x > _y) then {_y} else {_x};

// Should NOT trigger: using the built-in max function
private _correct = _x max _y;
