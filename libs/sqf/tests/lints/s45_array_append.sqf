// extending an array with a literal, reported
WMO_noRoadway = WMO_noRoadway + ["NonSteerable_Parachute_F", "RopeSegment"];

private _local = [1, 2];
_local = _local + [3];

// a different variable on the right, ignore
MY_array = OTHER_array + ["a"];

// prepending is not append, ignore
MY_array = ["a"] + MY_array;

// right hand side is not an array literal, could be a number or string, ignore
_total = _total + _delta;
_text = _text + "more";
MY_array = MY_array + OTHER_array;

// not an addition, ignore
MY_array = [1, 2, 3];
MY_array append ["a"];
