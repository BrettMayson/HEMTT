// Should trigger warning: if (x >= 0) then {x} else {-x}
private _x = 5;
private _abs1 = if (_x >= 0) then {_x} else {-_x};

// Should trigger warning: if (x < 0) then {-x} else {x}
private _abs2 = if (_x < 0) then {-_x} else {_x};

// Should trigger warning: if (x > 0) then {x} else {-x}
private _abs3 = if (_x > 0) then {_x} else {-_x};

// Should trigger warning: if (x <= 0) then {-x} else {x}
private _abs4 = if (_x <= 0) then {-_x} else {_x};

// Should NOT trigger: different variables
private _y = -3;
private _notAbs1 = if (_x >= 0) then {_x} else {-_y};

// Should NOT trigger: not comparing to 0
private _notAbs2 = if (_x >= 5) then {_x} else {-_x};

// Should NOT trigger: branches don't match pattern
private _notAbs3 = if (_x >= 0) then {_x * 2} else {-_x};

// Should NOT trigger: using the built-in abs function
private _correct = abs _x;
