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

// looks like 🥧 but it's just data in an array, should NOT trigger
data = ["land_house_k_3_ep1",[1.9079,2.71822,3.14022]];
data2 = ["land_house_k_3_ep1",[[-5.54529,-3.52778,-0.717676],[2.9529,5.68822,-0.299778],[1.9079,2.71822,3.14022],[-5.39259,4.35341,-0.89525],[-5.18259,3.54341,1.99825],[-2.42948,-0.243622,-0.716877],[-1.33908,5.83411,2.85086],[-2.42948,-0.243622,-0.716877]]];
