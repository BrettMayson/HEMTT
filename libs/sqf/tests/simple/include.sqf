#include "something.hpp"

#define VAR(var) thing##var

{
	private VAR(hi) = _x + "test";
} forEach [0,1,2,3];
systemChat str _things;
