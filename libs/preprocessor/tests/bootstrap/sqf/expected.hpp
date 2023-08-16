

if (42 == 42) then {
systemChat "The answer to life, the universe, and everything is 42";
} else {
systemChat "oops";
};


private _function = "test";

diag_log text format ['[%1] (%2) %3: %4', toUpper 'PREFIX', 'COMPONENT', '', format["Error calling %1: %2", _function, (str 42)]];
