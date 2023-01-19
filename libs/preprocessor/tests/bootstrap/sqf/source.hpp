#include "some_values.hpp"

if (MY_VALUE == 42) then {
	systemChat "The answer to life, the universe, and everything is 42";
} else {
	systemChat ERROR_MSG;
};

#define LOG_SYS_FORMAT(LEVEL,MESSAGE) format ['[%1] (%2) %3: %4', toUpper 'PREFIX', 'COMPONENT', LEVEL, MESSAGE]
#define LOG_SYS(LEVEL,MESSAGE) diag_log text LOG_SYS_FORMAT(LEVEL,MESSAGE)
#define ERROR(MESSAGE) LOG_SYS('ERROR',MESSAGE)
#define ERROR_2(MESSAGE,ARG1,ARG2) ERROR(FORMAT_2(MESSAGE,ARG1,ARG2))
#define FORMAT_2(STR,ARG1,ARG2) format[STR, ARG1, ARG2]

private _function = "test";

ERROR_2("Error calling %1: %2", _function, (str MY_VALUE));
