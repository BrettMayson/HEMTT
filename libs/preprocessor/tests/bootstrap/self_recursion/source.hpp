my_fnc = { systemChat "Yes!" };
#define my_fnc { systemChat "Maybe?"; call my_fnc }
call my_fnc;

#define ARR_2(ARG1,ARG2) ARG1, ARG2
#define QUOTE(ARG) #ARG

QUOTE([ARR_2('ace_infoDisplayChanged', [ARR_2(_this select 0, 'Any')])] call CBA_fnc_localEvent;);
