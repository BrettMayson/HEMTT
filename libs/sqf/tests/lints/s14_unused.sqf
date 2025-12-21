private _z = 5; // and never used
params ["_something"];

private _x = 10;
_x = 11; // overwriten before use
systemChat str _x; // later used once
