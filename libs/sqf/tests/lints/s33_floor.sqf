// Should trigger warning: x - (x % 1)
private _x = 5.7;
private _floor1 = _x - (_x % 1);

// Should trigger warning with different variable name
private _value = 3.14;
private _floor2 = _value - (_value % 1);

// Should NOT trigger: different variables
private _y = 2;
private _notFloor1 = _x - (_y % 1);

// Should NOT trigger: not modulo 1
private _notFloor2 = _x - (_x % 2);

// Should NOT trigger: using the built-in floor function
private _correct = floor _x;
