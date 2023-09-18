#define ADDON DOUBLES(PREFIX,COMPONENT)
#define DOUBLES(var1,var2) var1##_##var2

#define PREFIX hello
#define COMPONENT world

ADDON = "hello_world";
class RscDiary { // for loading saves use uiNamespace because missionNamespace is not restored before map is loaded
#ifdef NOTREAL
	ADDON = "hello_world";
#endif
};
