#define VALUE 1
#define NESTED VALUE
#define DOUBLES(var1,var2) var1##_##var2

DOUBLES(var2,var1)
DOUBLES(var1,var2)
DOUBLES(var1,NESTED)
