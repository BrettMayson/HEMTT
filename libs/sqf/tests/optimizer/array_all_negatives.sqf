// all arrays should NOT be convertible to statics

a = selectRandom [[]];   // too small (requires +op and only rel size 2)
{} forEach [];           // empty
