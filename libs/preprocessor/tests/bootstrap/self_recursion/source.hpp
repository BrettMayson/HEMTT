my_fnc = { systemChat "Yes!" };
#define my_fnc { systemChat "Maybe?"; call my_fnc }
call my_fnc;

#define ARR_2(ARG1,ARG2) ARG1, ARG2
#define QUOTE(ARG) #ARG

QUOTE([ARR_2('ace_infoDisplayChanged', [ARR_2(_this select 0, 'Any')])] call CBA_fnc_localEvent;);

#define GETVAR_SYS(var1,var2) getVariable [ARR_2(QUOTE(var1),var2)]
#define GETVAR(var1,var2,var3) (var1 GETVAR_SYS(var2,var3))
private _side = [west,east,independent,civilian] select (GETVAR(_display,newSide,GETVAR(_display,oldSide,0)));
