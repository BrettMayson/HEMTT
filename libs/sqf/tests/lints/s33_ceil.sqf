// Should trigger warning: x + (1 - (x % 1))
private _x = 5.3;
private _ceil1 = _x + (1 - (_x % 1));

// Should trigger warning: (1 - (x % 1)) + x (reversed operands)
private _ceil2 = (1 - (_x % 1)) + _x;

// Should trigger warning: (x - (x % 1)) + 1
private _ceil3 = (_x - (_x % 1)) + 1;

// Should trigger warning: 1 + (x - (x % 1)) (reversed operands)
private _ceil4 = 1 + (_x - (_x % 1));

// Should NOT trigger: different variables
private _y = 2;
private _notCeil1 = _x + (1 - (_y % 1));

// Should NOT trigger: not modulo 1
private _notCeil2 = _x + (1 - (_x % 2));

// Should NOT trigger: using the built-in ceil function
private _correct = ceil _x;
