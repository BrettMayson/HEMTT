private ["_a", "_b", "_c", "_d"];
_a = 1;
_b = 2;
_c = 3;
_d = 4;

private "_e";
_e = 5;

private ["_x"];
_x = 5;

private _y = 10;

private ["_z"];
[1,2,3] apply {
    _z = _z + _x;
};
