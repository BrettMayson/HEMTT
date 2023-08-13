my_fnc = { systemChat "Yes!" };
call { systemChat "Maybe?"; call my_fnc };


"['ace_infoDisplayChanged', [_this select 0, 'Any']] call CBA_fnc_localEvent;";

