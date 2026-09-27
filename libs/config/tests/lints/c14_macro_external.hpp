#define EXTERNAL(NAME) class NAME
#define QUOTE(var1) #var1
#define DOUBLES(var1,var2) var1##_##var2
#define PREFIX cba
#define EXT_PREFIXED(NAME) class DOUBLES(PREFIX,NAME)

class CfgVehicles {
    EXTERNAL(alpha);
    EXT_PREFIXED(beta);
    class gamma;
    class used_me;
    class child: used_me {};
};
