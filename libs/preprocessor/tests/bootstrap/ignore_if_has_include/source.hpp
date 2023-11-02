#define THING "yes"

#pragma hemtt flag pe23_ignore_has_include
#if __has_include("idk.hpp")
#undef THING
#define THING "no"
#endif

value = THING;
