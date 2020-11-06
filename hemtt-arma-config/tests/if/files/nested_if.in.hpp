#define TEST

#ifdef TEST
#define SKIP
#endif

#ifdef SKIP
#ifndef TEST
test = "false";
#else
test = "true";
#endif
skip = "true";
#else
skip = "false";
#endif

#ifndef SKIP
#ifndef TEST
test = "false";
#else
test = "true";
#endif
skip = "true";
#else
skip = "false";
#endif
