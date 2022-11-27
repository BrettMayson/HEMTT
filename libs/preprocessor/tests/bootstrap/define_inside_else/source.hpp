#define COMPONENT hello
#define COMPONENT_BEAUTIFIED Hello

#ifdef COMPONENT_BEAUTIFIED
	path = 1;
	#define COMPONENT_NAME QUOTE(hello - COMPONENT_BEAUTIFIED)
#else
	path = 2;
	#define COMPONENT_NAME QUOTE(hello - COMPONENT)
#endif

value = COMPONENT_NAME;
