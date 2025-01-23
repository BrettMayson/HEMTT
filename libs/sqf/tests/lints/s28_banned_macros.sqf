#define TEST_EXTRA_LOGGING

#ifdef TEST_EXTRA_LOGGING
hint format ["z is %1", z];
#endif


#define SOME_MACRO

#ifdef SOME_MACRO
x = 5;
#endif

