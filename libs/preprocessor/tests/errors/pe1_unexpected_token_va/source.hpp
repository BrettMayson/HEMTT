#define FUNC A
#define TRANSFORM(...) __VA_APPLY__(FUNC)
TRANSFORM(1,2,3) // 'A,A,A'
