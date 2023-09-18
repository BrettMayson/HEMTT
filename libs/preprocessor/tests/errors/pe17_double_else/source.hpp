#define test 1
#if test
value = 1;
#else
value = 0;
#endif

#if test
value = 1;
#else
value = 0;
#else
value = 2;
#endif
