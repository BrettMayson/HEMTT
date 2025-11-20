// Should trigger warning: if (x < y) then {x} else {y}
private _x = 5;
private _y = 10;
private _min1 = if (_x < _y) then {_x} else {_y};

// Should trigger warning: if (x <= y) then {x} else {y}
private _min2 = if (_x <= _y) then {_x} else {_y};

// Should trigger warning: if (x > y) then {y} else {x} (reversed)
private _min3 = if (_x > _y) then {_y} else {_x};

// Should trigger warning: if (x >= y) then {y} else {x} (reversed)
private _min4 = if (_x >= _y) then {_y} else {_x};

// Should trigger warning: [_x, _y] select (_x > _y)
private _min5 = [_x, _y] select (_x > _y);

// Should NOT trigger: different variables
private _z = 3;
private _notMin1 = if (_x < _y) then {_x} else {_z};

// Should NOT trigger: branches don't match pattern
private _notMin2 = if (_x < _y) then {_y} else {_x};

// Should NOT trigger: using the built-in min function
private _correct = _x min _y;
