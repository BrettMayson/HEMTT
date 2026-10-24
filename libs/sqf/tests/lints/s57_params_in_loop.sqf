params ["_outsideAnyLoop", "_values"];

{
    params ["_badForEach"];
    hint str _forEachIndex; // for S41 
} forEach _values;

_values apply {
    params ["_badApply"];
    _badApply + 1
};

{
    _x params ["_xValue"];
    hint str _forEachIndex; // for S41 
} forEach _values;

{
    [{
        params ["_differentContext"];
    }, [5], 6] call CBA_fnc_waitAndExecute;
    hint str _forEachIndex; // for S41 
} forEach _outsideAnyLoop;
