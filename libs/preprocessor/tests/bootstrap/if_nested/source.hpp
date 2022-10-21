#define TEST

#ifdef TEST
#define SKIP
#endif

#ifdef SKIP
#ifndef TEST
test = "hidden";
#else
test = "shown";
#endif
skip = "shown";
#else
skip = "hidden";
#endif

#ifndef SKIP
#ifndef TEST
test = "hidden";
#else
test = "hidden";
#endif
skip = "hidden";
#else
skip = "shown";
#endif
