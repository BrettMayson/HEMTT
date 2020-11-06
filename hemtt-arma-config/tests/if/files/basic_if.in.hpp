#define TEST 1

#ifdef TEST
test = "true";
#else
test = "false";
#endif

#undef TEST

#ifdef TEST
test = "true";
#else
test = "false";
#endif
