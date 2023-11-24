#include "script_component.hpp"

GVAR(test) = 1;
systemChat format ["%1 is %2", QGVAR(test), GVAR(test)];
