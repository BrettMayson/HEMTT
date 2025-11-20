// Should trigger warning: atan(y / x)
private _x = 5;
private _y = 10;
private _angle1 = atan(_y / _x);

// Should trigger warning with different variables
private _dx = 3;
private _dy = 4;
private _angle2 = atan(_dy / _dx);

// Should NOT trigger: atan without division
private _notAtan1 = atan _x;

// Should NOT trigger: using the built-in atan2 function
private _correct = _x atan2 _y;
