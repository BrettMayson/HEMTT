// all arrays should be convertible to statics

params ["_array"];
a = _array vectorAdd [1,2,3];
b = selectRandom [1,2,3];
c = [4,5,6] findIf code;
