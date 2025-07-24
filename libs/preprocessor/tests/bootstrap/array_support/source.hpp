#define VERSION_ARRAY(major, minor, patch) {major, minor, patch}
#define ARRAY_MACRO(name, arr) class name { version[] = arr; };

version = VERSION_ARRAY(1, 2, 3);
ARRAY_MACRO(test_mod, {3, 15, 7})