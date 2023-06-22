#include "some_macros.hpp"
#include "some_values.hpp"

if (MY_VALUE == 42) then {
	systemChat "The answer to life, the universe, and everything is 42";
} else {
	systemChat ERROR_MSG;
};

private _function = "test";

ERROR_2("Error calling %1: %2", _function, (str MY_VALUE));
