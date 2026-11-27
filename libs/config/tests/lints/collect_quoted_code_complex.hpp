// From ace_field_manual, complex macros and quotes in quotes
#define PREFIX abe
#define COMPONENT field

#define ADDON DOUBLES(PREFIX,COMPONENT)
#define MAIN_ADDON DOUBLES(PREFIX,main)
#define FUNC(var1) TRIPLES(ADDON,fnc,var1)
#define FUNCMAIN(var1) TRIPLES(PREFIX,fnc,var1)
#define FUNC_INNER(var1,var2) TRIPLES(DOUBLES(PREFIX,var1),fnc,var2)
#define ARR_1(ARG1) ARG1
#define ARR_2(ARG1,ARG2) ARG1, ARG2
#define DOUBLES(var1,var2) var1##_##var2
#define TRIPLES(var1,var2,var3) var1##_##var2##_##var3
#define QUOTE(x) #x
#define GVAR(var1) DOUBLES(ADDON,var1)
#define EGVAR(var1,var2) TRIPLES(PREFIX,var1,var2)
#define QGVAR(var1) QUOTE(GVAR(var1))
#define QEGVAR(var1,var2) QUOTE(EGVAR(var1,var2))
#define QQGVAR(var1) QUOTE(QGVAR(var1))
#define QQEGVAR(var1,var2) QUOTE(QEGVAR(var1,var2))

#define BASE_ARGUMENTS "'    '", \
QUOTE([ARR_2('ACE3 Common',QQEGVAR(interact_menu,selfInteractKey))] call FUNC(getKeybindName)), \
QUOTE([ARR_2('ACE3 Common',QQEGVAR(interact_menu,interactKey))] call FUNC(getKeybindName))

class CfgHints {
    class ACE_Items {
        class ACE_ATragMX {
            arguments[] = {BASE_ARGUMENTS, QUOTE('ACE_ATragMX' call FUNC(getItemName))};
        };
    };
};
