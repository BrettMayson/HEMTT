#define QUOTE(var1) #var1
#define DOUBLES(var1,var2) var1##_##var2
#define ADDON test
#define GVAR(var1) DOUBLES(ADDON,var1)
#define QGVAR(var1) QUOTE(GVAR(var1))
#define QQGVAR(var1) QUOTE(QGVAR(var1))

value = GVAR(myVar);
value = QGVAR(myVar);
value = QUOTE(My variable is QQGVAR(myVar));
