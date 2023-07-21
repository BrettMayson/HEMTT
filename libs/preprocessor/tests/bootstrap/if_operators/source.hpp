#define FLAG 1
#if FLAG
data = "matched flag";
#endif

#if __COUNTER == 0
data = "matched 0";
#endif

#if __COUNTER != 0
data = "matched not zero";
#else
data = "skipped";
#endif

#define A 1
#define B 2

#if B > A
data = "matched greater";
#endif

#if 2 > A
data = "matched greater 2";
#endif

#define COUNTER 0
#if COUNTER == 0
rank = "SERGEANT";
#else
#if COUNTER == 1
rank = "CORPORAL";
#else
rank = "PRIVATE";
#endif
#endif
