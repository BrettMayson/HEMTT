#define TEST1(var1, var2) INNER(var1, var2)
#define TEST2(var1, var2) INNER(var1, var2) // Separate macro to trigger unique warning for INNER padding
#define INNER(var1, var2) var1 var2

TEST1(John,Smith); // Only inner
TEST2(John, Smith); // Both call and inner
