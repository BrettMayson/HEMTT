test_fnc_A = {1};
["test_fnc_b", {2}] call cba_fnc_compilefinal;
["file", "test_fnc_C"] call cba_fnc_compilefunction;

[] call test_fnc_x;
[] spawn test_fnc_Y;

[] call a3_fnc_shuffle;
