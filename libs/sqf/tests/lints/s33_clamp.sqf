// Should trigger warning: if (_v < min) then {min} else { if (_v > max) then {max} else {_v} }
private _v = 5;
private _min = 0;
private _max = 10;
private _clamped1min = if (_v < _min) then {_min} else { if (_v > _max) then {_max} else {_v} };
private _clamped1max = if (_v > _max) then {_max} else { if (_v < _min) then {_min} else {_v} };

// Should trigger warning: with <= and >=
private _clamped2min = if (_v <= _min) then {_min} else { if (_v >= _max) then {_max} else {_v} };
private _clamped2max = if (_v >= _max) then {_max} else { if (_v <= _min) then {_min} else {_v} };

// Should trigger warning: command inside
private _clamped3min = if (_v < _min) then {_min} else { _v max _max };
private _clamped3max = if (_v > _max) then {_max} else { _v min _min };

// Should NOT trigger: different variables in branches
private _w = 7;
private _notClamped1 = if (_v < _min) then {_min} else { if (_w > _max) then {_max} else {_v} };

// Should NOT trigger: branches don't match pattern
private _notClamped2 = if (_v < _min) then {_min} else { if (_v > _max) then {_v} else {_max} };

// Should NOT trigger: missing nested if-then-else
private _notClamped3 = if (_v < _min) then {_min} else {_v};

// Should NOT trigger: using the correct clamp pattern
private _correct = _min max _v min _max;
