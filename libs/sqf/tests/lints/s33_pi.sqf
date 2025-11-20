// Should trigger warning: using manual pi values
private _x = 10;
private _result1 = _x / 3.14;
private _result2 = _x * 3.141;
private _result3 = _x + 3.1415;
private _result4 = _x - 3.14159;
private _result5 = _x / 3.141592;
private _result6 = _x * 3.1415926;
private _result7 = _x + 3.14159265;

// Should trigger: standalone value
private _pi1 = 3.14;
private _pi2 = 3.1415926;

// Should NOT trigger: not pi values
private _notPi1 = 3.13;
private _notPi2 = 3.15;
private _notPi3 = 3.1;
private _notPi4 = 3.2;
private _notPi5 = 3.0;
private _notPi6 = 4.14;

// Should NOT trigger: using the correct pi command
private _correct = _x / pi;
private _correct2 = sin(pi / 2);
