// Should trigger warning: x - y * floor(x / y)
private _x = 17;
private _y = 5;
private _mod1 = _x - _y * floor(_x / _y);

// Should trigger warning with different variables
private _a = 23;
private _b = 7;
private _mod2 = _a - _b * floor(_a / _b);

// Should NOT trigger: different variables don't match pattern
private _z = 3;
private _notMod1 = _x - _y * floor(_x / _z);

// Should NOT trigger: not using floor
private _notMod2 = _x - _y * (_x / _y);

// Should NOT trigger: using the built-in mod function
private _correct = _x mod _y;
