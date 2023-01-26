#define IS_ADMIN_SYS(x) x##kick
#define IS_ADMIN serverCommandAvailable 'IS_ADMIN_SYS(#)'

if (IS_ADMIN) exitWith {};
